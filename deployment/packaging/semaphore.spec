%global debug_package %{nil}
%global _missing_build_ids_terminate_build 0
%global _dwz_low_mem_die_limit 0

Name:           velum
Version:        2.8.90
Release:        1%{?dist}
Summary:        Velum UI is a modern UI for Ansible, Terraform, OpenTofu, Bash and Pulumi. It lets you easily run Ansible playbooks, get notifications about fails, control access to deployment system.

License:        MIT
URL:            https://github.com/tnl-o/velum
Source:         https://github.com/tnl-o/velum/archive/refs/tags/v2.8.90.zip

BuildRequires:  golang
BuildRequires:  nodejs
BuildRequires:  nodejs-npm
BuildRequires:  go-task
BuildRequires:  git
BuildRequires:  systemd-rpm-macros

Requires:       ansible

%description
Semaphore UI is a modern UI for Ansible, Terraform, OpenTofu, Bash and Pulumi. It lets you easily run Ansible playbooks, get notifications about fails, control access to deployment system.

%prep
%setup -q

%build
export SEMAPHORE_VERSION="development"
export SEMAPHORE_ARCH="linux_amd64"
export SEMAPHORE_CONFIG_PATH="./etc/semaphore"
export APP_ROOT="./semaphoreui/"

if ! [[ "$PATH" =~ "$HOME/go/bin:" ]]
then
    PATH="$HOME/go/bin:$PATH"
fi
export PATH
go-task all

cat > velumui.service <<EOF
[Unit]
Description=Velum Ansible
Documentation=https://github.com/tnl-o/velum
Wants=network-online.target
After=network-online.target

[Service]
Type=simple
ExecReload=/bin/kill -HUP $MAINPID
ExecStart=%{_bindir}/velum service --config=/etc/velum/config.json
SyslogIdentifier=velum
Restart=always

[Install]
WantedBy=multi-user.target

EOF

cat > velum-setup <<EOF
velum setup --config=/etc/velum/config.json
EOF

%install
mkdir -p %{buildroot}%{_sysconfdir}/velum/
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_unitdir}

install -m 755 bin/velum %{buildroot}%{_bindir}/velum
install -m 755 velum-setup %{buildroot}%{_bindir}/velum-setup
install -m 755 velumui.service %{buildroot}%{_unitdir}/velumui.service

%files
%license LICENSE
%doc README.md CONTRIBUTING.md
%attr(755, root, root) %{_bindir}/velum
%attr(755, root, root) %{_bindir}/velum-setup
%attr(644, root,root) %{_sysconfdir}/velum/
%{_unitdir}/velumui.service

%changelog
* Wed Jun 28 2023 Neftali Yagua
-
