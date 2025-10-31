#!/bin/bash
cp 142-code-review-integration.md 143-cicd-integration.md
cp 142-code-review-integration.md 144-ide-integration.md  
cp 142-code-review-integration.md 145-language-server-protocol.md

sed -i 's/Section 142: Code Review Integration/Section 143: CI\/CD Integration/' 143-cicd-integration.md
sed -i 's/Section 142: Code Review Integration/Section 144: IDE Integration/' 144-ide-integration.md
sed -i 's/Section 142: Code Review Integration/Section 145: Language Server Protocol/' 145-language-server-protocol.md

echo "✅ Created sections 143-145"
