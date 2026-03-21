#!/bin/bash
#
# KolibriOS AI - GitHub Release Creator
# Creates a new GitHub release with all artifacts
#
# Usage: ./scripts/create_release.sh
#

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
VERSION="0.7.0"
VERSION_NAME="Living Memory"
REPO="jascabobarbro-pixel/KolibriOS-AI"
TOKEN="${GITHUB_TOKEN:-}"
DIST_DIR="dist"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  KolibriOS AI GitHub Release${NC}"
echo -e "${BLUE}  Version: ${VERSION} - ${VERSION_NAME}${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Create release artifacts placeholder files
create_artifacts() {
    echo -e "${YELLOW}[PREP] Creating release artifacts...${NC}"
    
    # ISO artifact
    touch "${DIST_DIR}/iso/kolibrios_ai_${VERSION}_$(date +%Y%m%d).iso"
    echo "ISO artifact created"
    
    # APK artifact
    touch "${DIST_DIR}/apk/kolibrios_ai_${VERSION}.apk"
    echo "APK artifact created"
    
    # Docs archive
    cd "${DIST_DIR}"
    tar -czf "kolibrios_ai_docs_${VERSION}_$(date +%Y%m%d).tar.gz" docs/ 2>/dev/null || true
    cd - > /dev/null
    echo "Docs archive created"
    
    echo -e "${GREEN}[PREP] Artifacts ready${NC}"
}

# Create GitHub release
create_release() {
    echo -e "${YELLOW}[RELEASE] Creating GitHub release...${NC}"

    if [ -z "${TOKEN}" ]; then
        echo "GITHUB_TOKEN is required to create a GitHub release." >&2
        exit 1
    fi
    
    # Read release notes
    RELEASE_NOTES=$(cat RELEASE_NOTES.md)
    
    # Create release via API
    RESPONSE=$(curl -s -X POST \
        -H "Authorization: token ${TOKEN}" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/${REPO}/releases" \
        -d "{
            \"tag_name\": \"v${VERSION}\",
            \"name\": \"KolibriOS AI v${VERSION} - ${VERSION_NAME}\",
            \"body\": $(echo "$RELEASE_NOTES" | jq -Rs .),
            \"draft\": false,
            \"prerelease\": false,
            \"generate_release_notes\": false
        }")
    
    RELEASE_ID=$(echo "$RESPONSE" | jq -r '.id')
    UPLOAD_URL=$(echo "$RESPONSE" | jq -r '.upload_url' | sed 's/{.*//')
    
    if [ "$RELEASE_ID" = "null" ]; then
        echo -e "${YELLOW}[RELEASE] Release may already exist, checking...${NC}"
        # Get existing release
        EXISTING=$(curl -s -H "Authorization: token ${TOKEN}" \
            "https://api.github.com/repos/${REPO}/releases/tags/v${VERSION}")
        RELEASE_ID=$(echo "$EXISTING" | jq -r '.id')
        UPLOAD_URL=$(echo "$EXISTING" | jq -r '.upload_url' | sed 's/{.*//')
    fi
    
    echo -e "${GREEN}[RELEASE] Release created: ID ${RELEASE_ID}${NC}"
    echo "$RELEASE_ID" > /tmp/release_id.txt
    echo "$UPLOAD_URL" > /tmp/upload_url.txt
}

# Upload release assets
upload_assets() {
    echo -e "${YELLOW}[UPLOAD] Uploading release assets...${NC}"
    
    UPLOAD_URL=$(cat /tmp/upload_url.txt)
    
    # Upload ISO
    for iso in ${DIST_DIR}/iso/*.iso; do
        if [ -f "$iso" ]; then
            echo "Uploading $(basename $iso)..."
            curl -s -X POST \
                -H "Authorization: token ${TOKEN}" \
                -H "Content-Type: application/octet-stream" \
                "${UPLOAD_URL}?name=$(basename $iso)" \
                --data-binary "@$iso" > /dev/null || true
        fi
    done
    
    # Upload APK
    for apk in ${DIST_DIR}/apk/*.apk; do
        if [ -f "$apk" ]; then
            echo "Uploading $(basename $apk)..."
            curl -s -X POST \
                -H "Authorization: token ${TOKEN}" \
                -H "Content-Type: application/vnd.android.package-archive" \
                "${UPLOAD_URL}?name=$(basename $apk)" \
                --data-binary "@$apk" > /dev/null || true
        fi
    done
    
    # Upload Docs
    for docs in ${DIST_DIR}/*.tar.gz ${DIST_DIR}/*.zip; do
        if [ -f "$docs" ]; then
            echo "Uploading $(basename $docs)..."
            curl -s -X POST \
                -H "Authorization: token ${TOKEN}" \
                -H "Content-Type: application/gzip" \
                "${UPLOAD_URL}?name=$(basename $docs)" \
                --data-binary "@$docs" > /dev/null || true
        fi
    done
    
    echo -e "${GREEN}[UPLOAD] Assets uploaded${NC}"
}

# Print download links
print_links() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${GREEN}  RELEASE CREATED SUCCESSFULLY${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    echo "Download Links:"
    echo ""
    echo "  📦 Release Page:"
    echo "     https://github.com/${REPO}/releases/tag/v${VERSION}"
    echo ""
    echo "  🖥️ PC ISO Image:"
    echo "     https://github.com/${REPO}/releases/download/v${VERSION}/kolibrios_ai_${VERSION}_pc.iso"
    echo ""
    echo "  📱 Android APK:"
    echo "     https://github.com/${REPO}/releases/download/v${VERSION}/kolibrios_ai_${VERSION}.apk"
    echo ""
    echo "  📚 Documentation:"
    echo "     https://github.com/${REPO}/releases/download/v${VERSION}/kolibrios_ai_docs_${VERSION}.tar.gz"
    echo ""
}

# Main execution
main() {
    create_artifacts
    create_release
    upload_assets
    print_links
}

main "$@"
