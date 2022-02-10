# 개발 환경과 운영 환경을 분리
# branch로 구분하자
# 개발 환경일 때는 ./target/x86_64-unknown-linux-musl/release/madome-user
# 운영 환경일 때는 ./bin/$VERSION-linux-x86_64
# 이미지 버전도 개발 환경일 때는 latest, 운영 환경일 때는 $VERSION
# 운영 환경일 때는 도커 이미지 중에 해당 버전이 있을 경우에는 도커 빌드는 안함
# 개발 환경일 때는 무조건 함

# development mode
if [ "$(git branch --show-current)" = "release" ]; then
    VERSION="$(cat Cargo.toml | grep 'version = ' | head -1 | sed -e 's/version = //' | sed -e 's/\"//g')"

    if [ $? -ne 0 ]; then
        echo "failed parsing versio from Cargo.toml"
        exit 1
    fi

    BIN="./bin/linux-x86_64/$VERSION"

    if [ ! -f $BIN ]; then
        echo "binary file download"
        $PWD/update.sh
    fi

    if [ ! -f $BIN ]; then
        echo "binary file does not released or not found"
        exit 1
    fi
else
    # PREV_DOCKER_IMAGE_ID="$(docker images -q madome-user:latest)"

    cargo build --target=x86_64-unknown-linux-musl

    if [ $? -ne 0 ]; then
        exit 1
    fi

    VERSION="latest"

    BIN="./target/x86_64-unknown-linux-musl/debug/madome-user"
fi

chmod +x $BIN

POSTGRES_HOST="$(cat .env.release | grep "REAL_POSTGRES_HOST" | sed -e 's/REAL_POSTGRES_HOST=//')"

if [ $? -ne 0 ]; then
    echo "failed parsing REAL_POSTGRES_HOST from .env.release"
    exit 1
fi

docker build --build-arg BINARY_FILE="$BIN" --tag "madome-user:$VERSION" . --no-cache --rm --force-rm

if [ $? -ne 0 ]; then
    echo "failed docker build"
    exit 1
fi

cat k8s.yml | \
sed -e "s/{POSTGRES_HOST}/$POSTGRES_HOST/" | \
sed -e "s/{VERSION}/$VERSION/g" | \
sed -e "s%{WORK_DIR}%$PWD%g" | \
kubectl apply -f -

if [ $? -ne 0 ]; then
    echo "failed apply kubectl"
    exit 1
fi

if [ "$(git branch --show-current)" != "release" ]; then
    kubectl rollout restart deployment/madome-user
fi
