#!/bin/sh
set -eu

# On arm64 hosts (e.g. Apple Silicon under Docker Desktop), the Android SDK ships
# only x86_64 Linux binaries (aapt2, aapt, zipalign, ...). They run via Rosetta,
# but Rosetta needs the x86_64 glibc runtime present. Enable the amd64 package
# architecture and install the runtime libs aapt2 links against.
#
# No-op on amd64 hosts.

if [ "$(dpkg --print-architecture)" != "arm64" ]; then
  echo "setup-amd64-multiarch: host is $(dpkg --print-architecture), nothing to do."
  exit 0
fi

# amd64 packages are NOT on ports.ubuntu.com (arm64 mirror); they live on
# archive.ubuntu.com. Scope the existing arm64 source and add an amd64 one.
cat > /etc/apt/sources.list.d/ubuntu.sources <<'EOF'
Types: deb
URIs: http://ports.ubuntu.com/ubuntu-ports/
Suites: noble noble-updates noble-backports
Components: main universe restricted multiverse
Architectures: arm64
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg

Types: deb
URIs: http://ports.ubuntu.com/ubuntu-ports/
Suites: noble-security
Components: main universe restricted multiverse
Architectures: arm64
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg
EOF

cat > /etc/apt/sources.list.d/amd64.sources <<'EOF'
Types: deb
URIs: http://archive.ubuntu.com/ubuntu/
Suites: noble noble-updates noble-backports
Components: main universe restricted multiverse
Architectures: amd64
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg

Types: deb
URIs: http://security.ubuntu.com/ubuntu/
Suites: noble-security
Components: main universe restricted multiverse
Architectures: amd64
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg
EOF

dpkg --add-architecture amd64
apt-get update
apt-get install -y --no-install-recommends \
    libc6:amd64 \
    libstdc++6:amd64 \
    zlib1g:amd64
apt-get clean -y
rm -rf /var/lib/apt/lists/*
