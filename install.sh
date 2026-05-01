#!/usr/bin/env sh
set -eu

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
prefix=${PREFIX:-"$HOME/.local"}
bin_dir="$prefix/bin"

mkdir -p "$bin_dir"

cat > "$bin_dir/humanlint" <<EOF
#!/usr/bin/env sh
exec python3 "$repo_dir/tools/humanlint.py" "\$@"
EOF

chmod +x "$bin_dir/humanlint"
printf 'installed humanlint to %s\n' "$bin_dir/humanlint"
