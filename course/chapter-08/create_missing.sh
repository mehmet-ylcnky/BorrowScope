#!/bin/bash
# Copy template from existing section and modify
cp 93-frontend-project-setup.md 96-implementing-graph-view.md
cp 93-frontend-project-setup.md 98-graph-interactions.md
cp 93-frontend-project-setup.md 99-implementing-timeline-view.md

# Update titles
sed -i 's/Section 93: Frontend Project Setup/Section 96: Implementing Graph View/' 96-implementing-graph-view.md
sed -i 's/Section 93: Frontend Project Setup/Section 98: Graph Interactions/' 98-graph-interactions.md
sed -i 's/Section 93: Frontend Project Setup/Section 99: Implementing Timeline View/' 99-implementing-timeline-view.md

echo "Created sections 96, 98, 99"
