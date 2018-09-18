set -ex

main() {
    local target=
    if [ $TRAVIS_OS_NAME = linux ]; then
        target=x86_64-unknown-linux-musl
        sort=sort
    else
        target=x86_64-apple-darwin
        sort=gsort  # for `sort --sort-version`, from brew's coreutils.
    fi

    # Builds for iOS are done on OSX, but require the specific target to be
    # installed.
    case $TARGET in
        aarch64-apple-ios)
            rustup target install aarch64-apple-ios
            ;;
        armv7-apple-ios)
            rustup target install armv7-apple-ios
            ;;
        armv7s-apple-ios)
            rustup target install armv7s-apple-ios
            ;;
        i386-apple-ios)
            rustup target install i386-apple-ios
            ;;
        x86_64-apple-ios)
            rustup target install x86_64-apple-ios
            ;;
    esac

    # first we create a directory for the CMake binaries
    DEPS_DIR="${TRAVIS_BUILD_DIR}/deps"
    mkdir ${DEPS_DIR} && cd ${DEPS_DIR}
    # we use wget to fetch the cmake binaries
    travis_retry wget --no-check-certificate https://cmake.org/files/v3.3/cmake-3.3.2-Linux-x86_64.tar.gz
    # this is optional, but useful:
    # do a quick md5 check to ensure that the archive we downloaded did not get compromised
    echo "f3546812c11ce7f5d64dc132a566b749 *cmake-3.3.2-Linux-x86_64.tar.gz" > cmake_md5.txt
    md5sum -c cmake_md5.txt
    # extract the binaries; the output here is quite lengthy,
    # so we swallow it to not clutter up the travis console
    tar -xvf cmake-3.3.2-Linux-x86_64.tar.gz > /dev/null
    mv cmake-3.3.2-Linux-x86_64 cmake-install
    # add both the top-level directory and the bin directory from the archive
    # to the system PATH. By adding it to the front of the path we hide the
    # preinstalled CMake with our own.
    PATH=${DEPS_DIR}/cmake-install:${DEPS_DIR}/cmake-install/bin:$PATH
    # don't forget to switch back to the main build directory once you are done
    cd ${TRAVIS_BUILD_DIR}

    # This fetches latest stable release
    local tag=$(git ls-remote --tags --refs --exit-code https://github.com/japaric/cross \
                       | cut -d/ -f3 \
                       | grep -E '^v[0.1.0-9.]+$' \
                       | $sort --version-sort \
                       | tail -n1)
    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- \
           --force \
           --git japaric/cross \
           --tag $tag \
           --target $target
}

main
