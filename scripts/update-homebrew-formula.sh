#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "用法：$0 v0.1.0 YOUR_GITHUB_USERNAME"
  exit 1
fi

TAG="$1"
GITHUB_USERNAME="$2"
FORMULA="homebrew/autosort-mac.rb"
URL="https://github.com/${GITHUB_USERNAME}/autosort-mac/archive/refs/tags/${TAG}.tar.gz"

if [[ ! -f "${FORMULA}" ]]; then
  echo "错误：未找到 ${FORMULA}"
  exit 1
fi

TMP_FILE="$(mktemp)"
trap 'rm -f "${TMP_FILE}"' EXIT

echo "下载：${URL}"
curl -LfsS "${URL}" -o "${TMP_FILE}"

SHA256="$(shasum -a 256 "${TMP_FILE}" | awk '{print $1}')"

sed -i.bak \
  -e "s#homepage \".*\"#homepage \"https://github.com/${GITHUB_USERNAME}/autosort-mac\"#" \
  -e "s#url \".*\"#url \"${URL}\"#" \
  -e "s#sha256 \".*\"#sha256 \"${SHA256}\"#" \
  "${FORMULA}"
rm -f "${FORMULA}.bak"

echo "已更新 ${FORMULA}"
echo "sha256: ${SHA256}"
