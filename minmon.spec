Name:           minmon
Version:        0.9.1
Release:        1%{?dist}
Summary:        A minimalistic monitoring tool

License:        Apache 2.0, MIT
URL:            https://github.com/flo-at/minmon
Source0:        minmon-0.9.1.tar.gz

BuildRequires:  cargo
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
cargo build --release

%install
rm -rf %{buildroot}
install -D -m 0755 target/release/minmon %{buildroot}%{_bindir}/minmon
install -D -m 0644 systemd.minmon.service %{buildroot}%{_unitdir}/minmon.service

%files
%{_bindir}/minmon
%{_unitdir}/minmon.service


%changelog
* Wed Aug 28 2024 Ante de Baas <antedebaas@users.github.com> - 0.9.1-1
- Initial package