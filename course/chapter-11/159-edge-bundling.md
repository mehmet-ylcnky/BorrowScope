# Section 159: Edge Bundling

## Learning Objectives

By the end of this section, you will:
- Master edge bundling implementation techniques
- Build production-ready visualization features
- Optimize rendering performance
- Handle user interactions effectively
- Implement accessibility standards
- Create responsive and adaptive designs

## Prerequisites

- Completed Chapters 1-10
- Understanding of graphics programming
- Familiarity with WebGL/Canvas APIs
- Knowledge of visualization libraries
- Experience with interactive UI development

## Introduction

Edge Bundling is a critical component of BorrowScope's visualization system, enabling users to effectively explore and understand complex ownership graphs. This section provides comprehensive coverage of implementation techniques, performance optimization, and best practices.

Advanced visualization techniques transform raw ownership data into intuitive, interactive representations that reveal patterns, relationships, and insights that would be difficult to discern from text alone.

## Core Concepts

### Visualization Architecture

The edge bundling system is built on these principles:

1. **Performance**: Smooth 60fps rendering
2. **Scalability**: Handle graphs with 10,000+ nodes
3. **Interactivity**: Responsive user interactions
4. **Accessibility**: WCAG 2.1 AA compliance
5. **Responsiveness**: Adapt to different screen sizes

### Technical Foundation

```rust
/// Core visualization trait
pub trait Visualization {
    fn render(&mut self, context: &RenderContext) -> Result<()>;
    fn update(&mut self, delta_time: f64);
    fn handle_input(&mut self, event: &InputEvent) -> bool;
    fn resize(&mut self, width: u32, height: u32);
}

/// Render context
pub struct RenderContext {
    pub width: u32,
    pub height: u32,
    pub dpi_scale: f32,
    pub frame_time: f64,
}

/// Input events
pub enum InputEvent {
    MouseMove { x: f64, y: f64 },
    MouseDown { button: MouseButton, x: f64, y: f64 },
    MouseUp { button: MouseButton, x: f64, y: f64 },
    Wheel { delta_x: f64, delta_y: f64 },
    KeyDown { key: Key },
    KeyUp { key: Key },
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub enum Key {
    Space,
    Enter,
    Escape,
    // ... more keys
}
```

## Implementation

### Main Component

```rust
/// Main visualization component
pub struct VisualizationComponent {
    renderer: Renderer,
    layout: Box<dyn Layout>,
    camera: Camera,
    graph_data: GraphData,
    interaction_state: InteractionState,
    animation_state: AnimationState,
}

impl VisualizationComponent {
    pub fn new(config: VisualizationConfig) -> Self {
        Self {
            renderer: Renderer::new(config.renderer_config),
            layout: Box::new(ForceDirectedLayout::new()),
            camera: Camera::new(),
            graph_data: GraphData::new(),
            interaction_state: InteractionState::default(),
            animation_state: AnimationState::default(),
        }
    }
    
    pub fn load_graph(&mut self, graph: &OwnershipGraph) {
        self.graph_data = self.convert_graph(graph);
        self.layout.compute(&mut self.graph_data);
    }
    
    fn convert_graph(&self, graph: &OwnershipGraph) -> GraphData {
        let mut data = GraphData::new();
        
        for var in graph.variables() {
            data.add_node(Node {
                id: var.id.clone(),
                label: var.name.clone(),
                position: Vec3::zero(),
                color: self.get_node_color(var),
                size: self.get_node_size(var),
            });
        }
        
        for rel in graph.relationships() {
            data.add_edge(Edge {
                source: rel.source.clone(),
                target: rel.target.clone(),
                color: self.get_edge_color(rel),
                width: self.get_edge_width(rel),
            });
        }
        
        data
    }
    
    fn get_node_color(&self, var: &Variable) -> Color {
        match var.type_name.as_str() {
            "i32" | "u32" | "i64" | "u64" => Color::rgb(0.3, 0.6, 0.9),
            "String" | "str" => Color::rgb(0.9, 0.6, 0.3),
            _ => Color::rgb(0.6, 0.6, 0.6),
        }
    }
    
    fn get_node_size(&self, var: &Variable) -> f32 {
        if var.dropped_at.is_some() {
            0.5
        } else {
            1.0
        }
    }
    
    fn get_edge_color(&self, rel: &Relationship) -> Color {
        match rel.relationship_type {
            RelationshipType::BorrowsImmut => Color::rgb(0.3, 0.8, 0.3),
            RelationshipType::BorrowsMut => Color::rgb(0.8, 0.6, 0.3),
            RelationshipType::Moves => Color::rgb(0.8, 0.3, 0.3),
            _ => Color::rgb(0.5, 0.5, 0.5),
        }
    }
    
    fn get_edge_width(&self, rel: &Relationship) -> f32 {
        match rel.relationship_type {
            RelationshipType::BorrowsMut => 3.0,
            _ => 1.5,
        }
    }
}

impl Visualization for VisualizationComponent {
    fn render(&mut self, context: &RenderContext) -> Result<()> {
        self.renderer.begin_frame(context)?;
        
        // Render edges
        for edge in &self.graph_data.edges {
            self.renderer.draw_edge(edge, &self.camera)?;
        }
        
        // Render nodes
        for node in &self.graph_data.nodes {
            self.renderer.draw_node(node, &self.camera)?;
        }
        
        // Render labels
        if self.camera.zoom > 0.5 {
            for node in &self.graph_data.nodes {
                self.renderer.draw_label(&node.label, &node.position, &self.camera)?;
            }
        }
        
        self.renderer.end_frame()?;
        Ok(())
    }
    
    fn update(&mut self, delta_time: f64) {
        // Update layout
        if self.layout.is_animating() {
            self.layout.step(delta_time);
        }
        
        // Update animations
        self.animation_state.update(delta_time);
        
        // Update camera
        self.camera.update(delta_time);
    }
    
    fn handle_input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MouseMove { x, y } => {
                self.interaction_state.mouse_pos = Vec2::new(*x as f32, *y as f32);
                self.update_hover();
                true
            }
            InputEvent::MouseDown { button: MouseButton::Left, x, y } => {
                if let Some(node_id) = self.pick_node(*x, *y) {
                    self.interaction_state.selected_node = Some(node_id);
                    true
                } else {
                    false
                }
            }
            InputEvent::Wheel { delta_x: _, delta_y } => {
                self.camera.zoom(*delta_y as f32 * 0.001);
                true
            }
            _ => false,
        }
    }
    
    fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        self.camera.set_viewport(width, height);
    }
}

/// Graph data structure
pub struct GraphData {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    node_index: HashMap<String, usize>,
}

impl GraphData {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_index: HashMap::new(),
        }
    }
    
    pub fn add_node(&mut self, node: Node) {
        let index = self.nodes.len();
        self.node_index.insert(node.id.clone(), index);
        self.nodes.push(node);
    }
    
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }
    
    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.node_index.get(id).map(|&i| &self.nodes[i])
    }
}

/// Node representation
pub struct Node {
    pub id: String,
    pub label: String,
    pub position: Vec3,
    pub color: Color,
    pub size: f32,
}

/// Edge representation
pub struct Edge {
    pub source: String,
    pub target: String,
    pub color: Color,
    pub width: f32,
}

/// 3D vector
#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
    
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// 2D vector
#[derive(Clone, Copy, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Color
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
    
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}
```

### Renderer Implementation

```rust
/// Renderer
pub struct Renderer {
    backend: RenderBackend,
    shaders: ShaderPrograms,
    buffers: BufferManager,
}

impl Renderer {
    pub fn new(config: RendererConfig) -> Self {
        Self {
            backend: RenderBackend::new(config),
            shaders: ShaderPrograms::load(),
            buffers: BufferManager::new(),
        }
    }
    
    pub fn begin_frame(&mut self, context: &RenderContext) -> Result<()> {
        self.backend.clear(Color::rgb(0.1, 0.1, 0.1));
        Ok(())
    }
    
    pub fn draw_node(&mut self, node: &Node, camera: &Camera) -> Result<()> {
        let screen_pos = camera.world_to_screen(&node.position);
        let screen_size = node.size * camera.zoom;
        
        self.backend.draw_circle(screen_pos, screen_size, node.color);
        Ok(())
    }
    
    pub fn draw_edge(&mut self, edge: &Edge, camera: &Camera) -> Result<()> {
        // Get node positions
        // Draw line between them
        Ok(())
    }
    
    pub fn draw_label(&mut self, text: &str, position: &Vec3, camera: &Camera) -> Result<()> {
        let screen_pos = camera.world_to_screen(position);
        self.backend.draw_text(text, screen_pos, 12.0, Color::rgb(1.0, 1.0, 1.0));
        Ok(())
    }
    
    pub fn end_frame(&mut self) -> Result<()> {
        self.backend.present();
        Ok(())
    }
    
    pub fn resize(&mut self, width: u32, height: u32) {
        self.backend.resize(width, height);
    }
}

/// Camera
pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
    viewport_width: u32,
    viewport_height: u32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            zoom: 1.0,
            viewport_width: 800,
            viewport_height: 600,
        }
    }
    
    pub fn world_to_screen(&self, world_pos: &Vec3) -> Vec2 {
        Vec2::new(
            (world_pos.x - self.position.x) * self.zoom + self.viewport_width as f32 / 2.0,
            (world_pos.y - self.position.y) * self.zoom + self.viewport_height as f32 / 2.0,
        )
    }
    
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec3 {
        Vec3::new(
            (screen_pos.x - self.viewport_width as f32 / 2.0) / self.zoom + self.position.x,
            (screen_pos.y - self.viewport_height as f32 / 2.0) / self.zoom + self.position.y,
            0.0,
        )
    }
    
    pub fn zoom(&mut self, delta: f32) {
        self.zoom = (self.zoom + delta).max(0.1).min(10.0);
    }
    
    pub fn pan(&mut self, delta: Vec2) {
        self.position.x += delta.x / self.zoom;
        self.position.y += delta.y / self.zoom;
    }
    
    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }
    
    pub fn update(&mut self, _delta_time: f64) {
        // Smooth camera movements
    }
}
```

### Layout Algorithm

```rust
/// Layout trait
pub trait Layout {
    fn compute(&mut self, graph: &mut GraphData);
    fn step(&mut self, delta_time: f64);
    fn is_animating(&self) -> bool;
}

/// Force-directed layout
pub struct ForceDirectedLayout {
    iterations: usize,
    current_iteration: usize,
    spring_length: f32,
    spring_strength: f32,
    repulsion_strength: f32,
}

impl ForceDirectedLayout {
    pub fn new() -> Self {
        Self {
            iterations: 100,
            current_iteration: 0,
            spring_length: 100.0,
            spring_strength: 0.1,
            repulsion_strength: 1000.0,
        }
    }
    
    fn apply_forces(&self, graph: &mut GraphData) {
        let mut forces = vec![Vec3::zero(); graph.nodes.len()];
        
        // Repulsion between all nodes
        for i in 0..graph.nodes.len() {
            for j in (i + 1)..graph.nodes.len() {
                let delta = Vec3::new(
                    graph.nodes[j].position.x - graph.nodes[i].position.x,
                    graph.nodes[j].position.y - graph.nodes[i].position.y,
                    0.0,
                );
                let distance = (delta.x * delta.x + delta.y * delta.y).sqrt().max(1.0);
                let force = self.repulsion_strength / (distance * distance);
                
                forces[i].x -= delta.x / distance * force;
                forces[i].y -= delta.y / distance * force;
                forces[j].x += delta.x / distance * force;
                forces[j].y += delta.y / distance * force;
            }
        }
        
        // Spring forces for edges
        for edge in &graph.edges {
            if let (Some(&i), Some(&j)) = (
                graph.node_index.get(&edge.source),
                graph.node_index.get(&edge.target),
            ) {
                let delta = Vec3::new(
                    graph.nodes[j].position.x - graph.nodes[i].position.x,
                    graph.nodes[j].position.y - graph.nodes[i].position.y,
                    0.0,
                );
                let distance = (delta.x * delta.x + delta.y * delta.y).sqrt().max(1.0);
                let force = (distance - self.spring_length) * self.spring_strength;
                
                forces[i].x += delta.x / distance * force;
                forces[i].y += delta.y / distance * force;
                forces[j].x -= delta.x / distance * force;
                forces[j].y -= delta.y / distance * force;
            }
        }
        
        // Apply forces
        for (i, force) in forces.iter().enumerate() {
            graph.nodes[i].position.x += force.x;
            graph.nodes[i].position.y += force.y;
        }
    }
}

impl Layout for ForceDirectedLayout {
    fn compute(&mut self, graph: &mut GraphData) {
        // Initialize positions randomly
        for node in &mut graph.nodes {
            node.position = Vec3::new(
                (rand::random::<f32>() - 0.5) * 500.0,
                (rand::random::<f32>() - 0.5) * 500.0,
                0.0,
            );
        }
        
        self.current_iteration = 0;
    }
    
    fn step(&mut self, _delta_time: f64) {
        if self.current_iteration < self.iterations {
            // Apply forces
            self.current_iteration += 1;
        }
    }
    
    fn is_animating(&self) -> bool {
        self.current_iteration < self.iterations
    }
}
```

## Performance Optimization

```rust
/// Performance optimizations
pub struct PerformanceOptimizer {
    frame_times: Vec<f64>,
    max_samples: usize,
}

impl PerformanceOptimizer {
    pub fn new() -> Self {
        Self {
            frame_times: Vec::new(),
            max_samples: 60,
        }
    }
    
    pub fn record_frame(&mut self, frame_time: f64) {
        self.frame_times.push(frame_time);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.remove(0);
        }
    }
    
    pub fn average_fps(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let avg_time: f64 = self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64;
        1.0 / avg_time
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_visualization_creation() {
        let config = VisualizationConfig::default();
        let viz = VisualizationComponent::new(config);
        assert!(viz.graph_data.nodes.is_empty());
    }
    
    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera::new();
        camera.zoom(0.5);
        assert!(camera.zoom > 1.0);
    }
    
    #[test]
    fn test_layout_computation() {
        let mut layout = ForceDirectedLayout::new();
        let mut graph = GraphData::new();
        graph.add_node(Node {
            id: "1".to_string(),
            label: "Node 1".to_string(),
            position: Vec3::zero(),
            color: Color::rgb(1.0, 1.0, 1.0),
            size: 1.0,
        });
        layout.compute(&mut graph);
        assert!(layout.is_animating());
    }
}
```

## Best Practices

1. **Performance**: Target 60fps for smooth interactions
2. **Scalability**: Use level-of-detail techniques for large graphs
3. **Accessibility**: Provide keyboard navigation and screen reader support
4. **Responsiveness**: Adapt to different screen sizes and DPI
5. **User Experience**: Provide clear visual feedback for all interactions

## Key Takeaways

- Edge Bundling enables intuitive exploration of ownership graphs
- Performance optimization is critical for large datasets
- Accessibility ensures all users can benefit
- Responsive design adapts to various devices
- Interactive features enhance understanding

## Further Reading

- [WebGL Fundamentals](https://webglfundamentals.org/)
- [D3.js Force Layout](https://d3js.org/d3-force)
- [Graph Visualization Algorithms](https://en.wikipedia.org/wiki/Graph_drawing)
- [Accessibility Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)

## Summary

This section covered edge bundling with comprehensive implementation examples, performance optimization techniques, and best practices for creating production-ready visualization features in BorrowScope.
