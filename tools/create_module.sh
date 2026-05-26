#!/usr/bin/env bash
set -euo pipefail

module_name="${1:-}"
if [[ -z "$module_name" ]]; then
  echo "usage: ./tools/create_module.sh <module-name>" >&2
  exit 1
fi

crate_dir="crates/modules/$module_name"
if [[ -e "$crate_dir" ]]; then
  echo "module already exists: $module_name" >&2
  exit 1
fi

alias_name="${module_name//-/_}"
template_root="tools/templates/module"

mkdir -p \
  "$crate_dir/src/domain" \
  "$crate_dir/src/application" \
  "$crate_dir/src/infrastructure" \
  "$crate_dir/src/presentation/http"

cp "$template_root/Cargo.toml.tpl" "$crate_dir/Cargo.toml"
cp "$template_root/src/lib.rs.tpl" "$crate_dir/src/lib.rs"
cp "$template_root/src/domain/mod.rs.tpl" "$crate_dir/src/domain/mod.rs"
cp "$template_root/src/application/mod.rs.tpl" "$crate_dir/src/application/mod.rs"
cp "$template_root/src/infrastructure/mod.rs.tpl" "$crate_dir/src/infrastructure/mod.rs"
cp "$template_root/src/presentation/mod.rs.tpl" "$crate_dir/src/presentation/mod.rs"
cp "$template_root/src/presentation/http/mod.rs.tpl" "$crate_dir/src/presentation/http/mod.rs"
cp "$template_root/src/presentation/http/handlers.rs.tpl" "$crate_dir/src/presentation/http/handlers.rs"
cp "$template_root/src/presentation/http/routes.rs.tpl" "$crate_dir/src/presentation/http/routes.rs"

for file in \
  "$crate_dir/Cargo.toml" \
  "$crate_dir/src/lib.rs" \
  "$crate_dir/src/domain/mod.rs" \
  "$crate_dir/src/application/mod.rs" \
  "$crate_dir/src/infrastructure/mod.rs" \
  "$crate_dir/src/presentation/mod.rs" \
  "$crate_dir/src/presentation/http/mod.rs" \
  "$crate_dir/src/presentation/http/handlers.rs" \
  "$crate_dir/src/presentation/http/routes.rs"
do
  sed -i "s/__MODULE__/$module_name/g" "$file"
  sed -i "s/__MODULE_ALIAS__/$alias_name/g" "$file"
done

if ! grep -q "\"crates/modules/$module_name\"" Cargo.toml; then
  awk -v module="$module_name" '
    BEGIN { inserted = 0 }
    /^members = \[/ { print; in_members = 1; next }
    in_members && /^\]$/ && !inserted {
      print "  \"crates/modules/" module "\","
      inserted = 1
    }
    in_members && /^\]$/ { print; in_members = 0; next }
    { print }
  ' Cargo.toml > Cargo.toml.tmp
  mv Cargo.toml.tmp Cargo.toml
fi

if ! grep -q "^$module_name = " apps/api/Cargo.toml; then
  awk -v module="$module_name" '
    BEGIN { inserted = 0 }
    /^shared-kernel = / && !inserted {
      print
      print module " = { path = \"../../crates/modules/" module "\" }"
      inserted = 1
      next
    }
    { print }
  ' apps/api/Cargo.toml > apps/api/Cargo.toml.tmp
  mv apps/api/Cargo.toml.tmp apps/api/Cargo.toml
fi

if ! grep -q "${alias_name}_routes" apps/api/src/router.rs; then
  sed -i "s#use identity::presentation::http::routes as identity_routes;#use identity::presentation::http::routes as identity_routes;\nuse ${module_name}::presentation::http::routes as ${alias_name}_routes;#" apps/api/src/router.rs
  sed -i "s#        \\.merge(identity_routes::router())#        .merge(identity_routes::router())\n        .merge(${alias_name}_routes::router())#" apps/api/src/router.rs
fi
