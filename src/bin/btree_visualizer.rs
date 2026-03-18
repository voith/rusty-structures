use data_structures::btree::{BTree, BTreeNodeSnapshot, BTreeSnapshot};
use eframe::egui::{
    self, Align2, CentralPanel, Color32, FontId, Id, Pos2, Rect, ScrollArea, Sense, Stroke,
    StrokeKind, TextEdit, TopBottomPanel, Ui, Vec2,
};

const NODE_HEIGHT: f32 = 34.0;
const NODE_HORIZONTAL_PADDING: f32 = 14.0;
const HORIZONTAL_GAP: f32 = 30.0;
const VERTICAL_GAP: f32 = 88.0;
const CANVAS_PADDING: f32 = 40.0;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "B-Tree Visualizer",
        options,
        Box::new(|_cc| Ok(Box::new(BTreeVisualizerApp::default()))),
    )
}

struct BTreeVisualizerApp {
    tree: BTree<i32, String>,
    insert_key: String,
    insert_value: String,
    get_key: String,
    status: String,
}

impl Default for BTreeVisualizerApp {
    fn default() -> Self {
        Self {
            tree: BTree::new(2),
            insert_key: String::new(),
            insert_value: String::new(),
            get_key: String::new(),
            status: "Insert a key/value pair to start building the tree.".to_string(),
        }
    }
}

impl eframe::App for BTreeVisualizerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.heading("B-Tree Visualizer");
            ui.label(format!(
                "Minimum degree: {} | Keys stored: {}",
                self.tree.min_degree(),
                self.tree.len()
            ));
            ui.separator();
            controls(ui, self);
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.label(&self.status);
            ui.separator();

            let snapshot = self.tree.snapshot();
            ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    render_tree(ui, &snapshot);
                });
        });
    }
}

fn controls(ui: &mut Ui, app: &mut BTreeVisualizerApp) {
    ui.horizontal(|ui| {
        ui.label("Insert key");
        ui.add(TextEdit::singleline(&mut app.insert_key).desired_width(80.0));
        ui.label("Value");
        ui.add(TextEdit::singleline(&mut app.insert_value).desired_width(160.0));

        if ui.button("Insert").clicked() {
            match app.insert_key.trim().parse::<i32>() {
                Ok(key) => {
                    let value = app.insert_value.trim().to_string();
                    let previous = app.tree.insert(key, value.clone());
                    app.status = match previous {
                        Some(old_value) => format!(
                            "Updated key {key}. Previous value: {old_value}. New value: {value}."
                        ),
                        None => format!("Inserted key {key} with value {value}."),
                    };
                    app.insert_key.clear();
                    app.insert_value.clear();
                }
                Err(_) => {
                    app.status = "Insert key must be a valid i32.".to_string();
                }
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label("Get key");
        ui.add(TextEdit::singleline(&mut app.get_key).desired_width(80.0));

        if ui.button("Get").clicked() {
            match app.get_key.trim().parse::<i32>() {
                Ok(key) => match app.tree.get(&key) {
                    Some(value) => {
                        app.status = format!("Key {key} maps to value {value}.");
                    }
                    None => {
                        app.status = format!("Key {key} is not present in the tree.");
                    }
                },
                Err(_) => {
                    app.status = "Get key must be a valid i32.".to_string();
                }
            }
        }
    });
}

fn render_tree(ui: &mut Ui, snapshot: &BTreeSnapshot) {
    let Some(root) = &snapshot.root else {
        ui.label("The tree is empty.");
        return;
    };

    let measured = measure_node(root);
    let canvas_size = Vec2::new(
        measured.subtree_width + CANVAS_PADDING * 2.0,
        measured.subtree_height + CANVAS_PADDING * 2.0,
    );
    let (rect, _) = ui.allocate_exact_size(canvas_size, Sense::hover());
    let painter = ui.painter_at(rect);

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    place_node(
        root,
        Pos2::new(rect.min.x + CANVAS_PADDING, rect.min.y + CANVAS_PADDING),
        &mut nodes,
        &mut edges,
    );

    for edge in edges {
        painter.line_segment(
            [edge.from, edge.to],
            Stroke::new(1.5, Color32::from_rgb(90, 90, 90)),
        );
    }

    for node in nodes {
        let response = ui.interact(node.rect, Id::new(("btree-node", node.id)), Sense::hover());
        let fill = if response.hovered() {
            Color32::from_rgb(255, 244, 214)
        } else {
            Color32::from_rgb(234, 239, 244)
        };

        painter.rect(
            node.rect,
            8.0,
            fill,
            Stroke::new(1.0, Color32::from_rgb(52, 73, 94)),
            StrokeKind::Outside,
        );
        painter.text(
            node.rect.center(),
            Align2::CENTER_CENTER,
            node.label,
            FontId::monospace(14.0),
            Color32::from_rgb(30, 30, 30),
        );

        if response.hovered() {
            response.on_hover_ui(|ui| {
                ui.label(format!("Depth: {}", node.metadata.depth));
                ui.label(format!("Leaf: {}", node.metadata.is_leaf));
                ui.label(format!("Keys in node: {}", node.metadata.key_count));
                ui.label(format!("Children: {}", node.metadata.child_count));
            });
        }
    }
}

#[derive(Clone)]
struct MeasuredNode {
    subtree_width: f32,
    subtree_height: f32,
    node_width: f32,
    child_offsets: Vec<f32>,
    children: Vec<MeasuredNode>,
}

#[derive(Clone)]
struct DrawNode {
    id: usize,
    rect: Rect,
    label: String,
    metadata: NodeMetadata,
}

#[derive(Clone)]
struct NodeMetadata {
    depth: usize,
    is_leaf: bool,
    key_count: usize,
    child_count: usize,
}

#[derive(Clone)]
struct Edge {
    from: Pos2,
    to: Pos2,
}

fn measure_node(node: &BTreeNodeSnapshot) -> MeasuredNode {
    let label = node_label(node);
    let node_width = label.chars().count() as f32 * 8.0 + NODE_HORIZONTAL_PADDING * 2.0;

    if node.children.is_empty() {
        return MeasuredNode {
            subtree_width: node_width,
            subtree_height: NODE_HEIGHT,
            node_width,
            child_offsets: Vec::new(),
            children: Vec::new(),
        };
    }

    let children: Vec<MeasuredNode> = node.children.iter().map(measure_node).collect();
    let total_children_width = children
        .iter()
        .map(|child| child.subtree_width)
        .sum::<f32>()
        + HORIZONTAL_GAP * (children.len().saturating_sub(1) as f32);
    let subtree_width = node_width.max(total_children_width);

    let mut child_offsets = Vec::with_capacity(children.len());
    let mut current_x = (subtree_width - total_children_width) / 2.0;
    let mut max_child_height: f32 = 0.0;

    for child in &children {
        child_offsets.push(current_x);
        current_x += child.subtree_width + HORIZONTAL_GAP;
        max_child_height = max_child_height.max(child.subtree_height);
    }

    MeasuredNode {
        subtree_width,
        subtree_height: NODE_HEIGHT + VERTICAL_GAP + max_child_height,
        node_width,
        child_offsets,
        children,
    }
}

fn place_node(
    node: &BTreeNodeSnapshot,
    origin: Pos2,
    nodes: &mut Vec<DrawNode>,
    edges: &mut Vec<Edge>,
) {
    let measured = measure_node(node);
    place_node_with_measurement(node, &measured, origin, nodes, edges);
}

fn place_node_with_measurement(
    node: &BTreeNodeSnapshot,
    measured: &MeasuredNode,
    origin: Pos2,
    nodes: &mut Vec<DrawNode>,
    edges: &mut Vec<Edge>,
) {
    let node_min = Pos2::new(
        origin.x + (measured.subtree_width - measured.node_width) / 2.0,
        origin.y,
    );
    let node_rect = Rect::from_min_size(node_min, Vec2::new(measured.node_width, NODE_HEIGHT));

    nodes.push(DrawNode {
        id: node.id,
        rect: node_rect,
        label: node_label(node),
        metadata: NodeMetadata {
            depth: node.depth,
            is_leaf: node.is_leaf,
            key_count: node.key_count,
            child_count: node.child_count,
        },
    });

    for ((child, child_measured), child_offset) in node
        .children
        .iter()
        .zip(measured.children.iter())
        .zip(measured.child_offsets.iter())
    {
        let child_origin = Pos2::new(
            origin.x + *child_offset,
            origin.y + NODE_HEIGHT + VERTICAL_GAP,
        );
        let child_rect_min_x =
            child_origin.x + (child_measured.subtree_width - child_measured.node_width) / 2.0;
        let child_top = Pos2::new(
            child_rect_min_x + child_measured.node_width / 2.0,
            child_origin.y,
        );

        edges.push(Edge {
            from: Pos2::new(node_rect.center().x, node_rect.max.y),
            to: child_top,
        });

        place_node_with_measurement(child, child_measured, child_origin, nodes, edges);
    }
}

fn node_label(node: &BTreeNodeSnapshot) -> String {
    node.keys
        .iter()
        .zip(node.values.iter())
        .map(|(key, value)| format!("{key}:{value}"))
        .collect::<Vec<_>>()
        .join(" | ")
}
