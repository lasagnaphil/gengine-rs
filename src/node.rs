use storage::{Storage, ResourceID};
use sprite::SpriteData;
use sprite_renderer::SpriteRenderer;
use cgmath::Vector2;
use canvas::TileMap;

pub struct Script;

#[repr(C)]
pub struct NodeID {
    index: u32,
    generation: u16,
    ntid: u16,
}

pub trait Node {
    fn base(&self) -> &NodeBase;
}

pub trait NodeStorageBase {
    fn get<T>(id: NodeID) -> T;
}

pub struct NodeStorage<T: Node>;
impl<T> NodeStorage<T> where T: Node {
    pub fn new(size: u32) -> Self { NodeStorage }
}
impl<T> NodeStorageBase for NodeStorage<T> where T: Node {
    fn get<U>(id: NodeID) -> U { U::default() }
}

pub struct NodeBase {
    script: Option<ResourceID<Script>>,
    parent: NodeID,
    children: Vec<NodeID>,
}

pub trait Updateable {
    fn update(&self, dt: f32);
}

pub trait Drawable {
    fn draw(&self, renderer: &SpriteRenderer);
}

#[proc_macro_derive(Node)]
pub fn node_macro_derive(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_node_macro(&ast);
    gen.parse().unwrwap()
}

fn impl_node_macro(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    quote! {
        impl Node for #name {
            fn base(&self) -> &NodeBase { &self.base }
        }
    }
}

// Example Nodes

#[derive(Node)]
pub struct SpriteNode {
    pos: Vector2<f32>,
    sprite: ResourceID<SpriteData>,
    base: NodeBase
}

impl Drawable for SpriteNode {
    fn draw(&self, renderer: &SpriteRenderer) {
        renderer.draw_sprite_simple(self.sprite, self.pos, None);
    }
}

#[derive(Node)]
pub struct PlayerNode {
    sprite: Sprite,
    vel: vel,
    base: NodeBase,
}

impl Drawable for PlayerNode {
    fn draw(&self, renderer: &SpriteRenderer) {
        sprite.draw(renderer);
    }
}

#[derive(Node)]
pub struct TileMapNode {
    tilemap: TileMap,
    base: NodeBase
}

impl Updateable for TileMapNode {
    fn update(&self, dt: f32) {
        self.tilemap.update();
    }
}

struct GameWorld<'a> {
    sprite_nodes: NodeStorage<SpriteNode>,
    player_nodes: NodeStorage<PlayerNode>,
    tilemap_nodes: NodeStorage<TileMapNode>,
    // Note we need self-referencial struct support for this
    storages: Vec<&'a NodeStorageBase>
}

impl GameWorld {
    fn new() -> Self {
        let sprite_nodes = NodeStorage::<SpriteNode>::new(1024);
        let player_nodes = NodeStorage::<PlayerNode>::new(4);
        let tilemap_nodes = NodeStorage::<TileMapNode>::new(16);
        GameWorld {
            sprite_nodes, player_nodes, tilemap_nodes,
            storages: vec![
                &sprite_nodes as &NodeStorageBase,
                &player_nodes as &NodeStorageBase,
                &tilemap_nodes as &NodeStorageBase
            ]
        }
    }
    fn get_node<T: Node>(&self, id: NodeID) -> &T {
        self.storages[id.ntid].get::<T>(id)
    }
}
