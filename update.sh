if [ "$(git branch --show-current)" = "release" ]; then
    git pull

    if [ $? -ne 0 ]; then
        echo "failed git pull";
        exit 1
    fi

    VERSION="$(cat Cargo.toml | grep 'version = ' | head -1 | sed -e 's/version = //' | sed -e 's/\"//g')"

    github-release -v download \
        --user syrflover \
        --repo user.madome.app \
        --tag "v${VERSION}" \
        --name "madome-user-linux-x86_64"

    if [ $? -ne 0 ]; then
        echo "failed download from release"
        exit 1
    fi

    mkdir -p ./bin

    mv "./madome-user-linux-x86_64" "./bin/linux-x86_64/$VERSION"

    echo "succeed download"
else
    echo "not release branch"
fi