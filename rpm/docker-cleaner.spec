Name:           docker-cleaner
Version:        0.0.0
Release:        1%{?dist}
Summary:        Docker cleaner utility

License:        MIT
URL:            https://github.com/yourusername/docker-cleaner
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust cargo

%description
A tool to clean up dangling Docker files and directories.

%prep
%autosetup

%build
cargo build --release

%install
rm -rf $RPM_BUILD_ROOT
install -D -m 755 target/release/%{name} $RPM_BUILD_ROOT%{_bindir}/%{name}

%files
%{_bindir}/%{name}

%changelog
* Wed Jul 27 2024 Your Name <your.email@example.com> - 0.1.0-1
- Initial release
