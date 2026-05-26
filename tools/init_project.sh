#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: ./tools/init_project.sh <project-name>" >&2
  exit 1
}

die() {
  echo "$1" >&2
  exit 1
}

project_name="${1:-}"
[[ -n "$project_name" ]] || usage

case "$project_name" in
  */*)
    die "project name must not contain /"
    ;;
  "."|"..")
    die "project name is invalid: $project_name"
    ;;
esac

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source_root="$(cd "$script_dir/.." && pwd)"
parent_dir="$(cd "$source_root/.." && pwd)"
target_dir="$parent_dir/$project_name"

if [[ -e "$target_dir" ]]; then
  die "target already exists: $target_dir"
fi

project_snake="${project_name//-/_}"
project_title="$(
  printf '%s' "$project_name" |
    tr '_-' '  ' |
    awk '{
      for (i = 1; i <= NF; i++) {
        token = $i
        $i = toupper(substr(token, 1, 1)) tolower(substr(token, 2))
      }
      print
    }'
)"

escape_perl_replacement() {
  printf '%s' "$1" | perl -pe 's/([\\&\$])/\\$1/g'
}

project_name_repl="$(escape_perl_replacement "$project_name")"
project_snake_repl="$(escape_perl_replacement "$project_snake")"
project_title_repl="$(escape_perl_replacement "$project_title")"

cp -a "$source_root" "$target_dir"
rm -rf "$target_dir/.git" "$target_dir/target"

cargo_target_dir="$(mktemp -d /tmp/axum-init-target.XXXXXX)"
trap 'rm -rf "$cargo_target_dir"' EXIT

rewrite_file() {
  local file="$1"
  perl -0pi -e "s/axum-boilerplate/$project_name_repl/g; s/axum_boilerplate/$project_snake_repl/g; s/Axum Boilerplate/$project_title_repl/g; s/POSTGRES_DB=axum_boilerplate/POSTGRES_DB=$project_name_repl/g; s#DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/axum_boilerplate#DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/$project_name_repl#g; s/RUST_LOG=info,axum_boilerplate=debug/RUST_LOG=info,$project_snake_repl=debug/g" "$file"
}

while IFS= read -r -d '' file; do
  rewrite_file "$file"
done < <(find "$target_dir" -type f -print0)

cd "$target_dir"
cargo generate-lockfile
CARGO_TARGET_DIR="$cargo_target_dir" cargo fmt --all --check
CARGO_TARGET_DIR="$cargo_target_dir" cargo test --workspace --no-run

if rg -n --hidden --glob '!target/**' --glob '!.git/**' 'axum-boilerplate|axum_boilerplate|Axum Boilerplate' .; then
  die "legacy template slug still present"
fi
