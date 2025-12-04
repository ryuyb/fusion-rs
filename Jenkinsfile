// Requires a Jenkins credential (ID: ghcr-creds) containing GHCR username + token.
pipeline {
    agent any

    environment {
        REGISTRY = 'ghcr.io'
        DOCKER_CLI_EXPERIMENTAL = 'enabled'
    }

    options {
        timestamps()
        skipDefaultCheckout(true)
    }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Prepare Metadata') {
            steps {
                script {
                    def remote = sh(
                        script: "git config --get remote.origin.url",
                        returnStdout: true
                    ).trim()
                    def repo = remote
                        .replaceFirst(/.*github.com[:/]/, '')
                        .replaceFirst(/\.git$/, '')
                    env.IMAGE_NAME = repo

                    def branch = env.BRANCH_NAME ?: 'local'
                    def sanitizedBranch = branch.replaceAll('[^A-Za-z0-9_.-]', '-').toLowerCase()
                    def shortSha = sh(
                        script: 'git rev-parse --short HEAD',
                        returnStdout: true
                    ).trim()

                    def tags = [] as Set
                    tags << sanitizedBranch
                    if (env.TAG_NAME) {
                        tags << env.TAG_NAME
                    }
                    if (sanitizedBranch == 'main') {
                        tags << 'latest'
                    }
                    tags << "${sanitizedBranch}-sha-${shortSha}"

                    env.IMAGE = "${REGISTRY}/${repo}"
                    env.DOCKER_TAG_ARGS = tags.collect { "-t ${env.IMAGE}:${it}" }.join(' ')
                    echo "Will push tags: ${tags.join(', ')}"
                }
            }
        }

        stage('Set up Buildx') {
            steps {
                sh '''
                docker run --privileged --rm tonistiigi/binfmt --install all
                docker buildx create --name fusion-builder --use || docker buildx use fusion-builder
                docker buildx inspect --bootstrap
                '''
            }
        }

        stage('Login to GHCR') {
            steps {
                withCredentials([
                    usernamePassword(
                        credentialsId: 'ghcr-creds',
                        usernameVariable: 'GHCR_USER',
                        passwordVariable: 'GHCR_TOKEN'
                    )
                ]) {
                    sh 'echo "$GHCR_TOKEN" | docker login ${REGISTRY} -u "$GHCR_USER" --password-stdin'
                }
            }
        }

        stage('Build and Push') {
            steps {
                sh """
                docker buildx build \\
                  --platform linux/amd64,linux/arm64 \\
                  ${env.DOCKER_TAG_ARGS} \\
                  --push \\
                  .
                """
            }
        }
    }

    post {
        always {
            sh 'docker buildx rm fusion-builder || true'
        }
    }
}
