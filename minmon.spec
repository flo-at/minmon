Name:           minmon
Version:        0.9.1
Release:        1%{?dist}
Summary:        A minimalistic monitoring tool

License:        Apache 2.0, MIT
URL:            https://github.com/flo-at/minmon
Source0:        minmon-0.9.1.tar.gz

BuildRequires:  cargo
BuildRequires:  openssl-devel
BuildRequires:  lm_sensors-devel

Requires:       glibc
Requires:       openssl-libs
Requires:       libgcc
Requires:       lm_sensors-libs
Requires:       zlib

Provides:       minmon = 0.9.1
Provides:       minmon(x86_64) = 0.9.1

%description
A minimalistic monitoring tool.

%license %{name}/LICENSE-APACHE
%license %{name}/LICENSE-MIT

%global debug_package %{nil}
%prep
%setup -q

%build
export RUSTFLAGS="-C debuginfo=2"
cargo build --release --features full

%install
rm -rf %{buildroot}
install -D -m 0755 target/release/minmon %{buildroot}%{_bindir}/minmon
mkdir -p %{buildroot}/etc/minmon
install -m 644 minmon.toml %{buildroot}/etc/minmon/minmon.toml
ln -s /etc/minmon/minmon.toml %{buildroot}/etc/minmon.toml
install -D -m 0644 systemd.minmon.service %{buildroot}%{_unitdir}/minmon.service

%files
%{_bindir}/minmon
%{_unitdir}/minmon.service
%config(noreplace) /etc/minmon.toml
%config(noreplace) /etc/minmon/minmon.toml

%changelog
* Wed Aug 28 2024 Ante de Baas <antedebaas@users.github.com> - 0.9.1-1
- Initial package