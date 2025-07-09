use crate::{metadata::MetaData, sys::aiNode, *};

use derivative::Derivative;

use std::sync::{Arc, Weak};

#[derive(Default, Derivative)]
#[derivative(Debug)]
pub struct Node {
    pub name: String,
    pub children: Arc<[Arc<Node>]>,
    pub meshes: Vec<u32>,
    pub metadata: Option<MetaData>,
    pub transformation: Matrix4x4,
    #[derivative(Debug = "ignore")]
    pub parent: Weak<Node>,
}

impl Node {
    pub(crate) fn new(node: &aiNode) -> Arc<Node> {
        Self::allocate(node, None)
    }

    fn allocate(node: &aiNode, parent: Option<Weak<Node>>) -> Arc<Node> {
        let current_node = Arc::new_cyclic(|weak_self| {
            let children = utils::get_base_type_vec_from_raw(node.mChildren, node.mNumChildren)
                .into_iter()
                .map(|child| Self::allocate(child, Some(weak_self.clone())))
                .collect::<Vec<_>>()
                .into();

            Node {
                name: node.mName.into(),
                children,
                meshes: utils::get_raw_vec(node.mMeshes, node.mNumMeshes),
                metadata: utils::get_raw(node.mMetaData),
                transformation: (&node.mTransformation).into(),
                parent: parent.unwrap_or_else(Weak::new),
            }
        });

        current_node
    }
}

#[cfg(test)]
mod test {
    use crate::utils;

    #[test]
    fn checking_nodes() {
        use crate::scene::{PostProcess, Scene};

        let current_directory_buf = utils::get_model("models/BLEND/box.blend");

        let scene = Scene::from_file(
            current_directory_buf.as_str(),
            vec![
                PostProcess::CalculateTangentSpace,
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
            ],
        )
        .unwrap();

        let root = scene.root.as_ref().unwrap();
        let children = &root.children;

        assert_eq!("<BlenderRoot>".to_string(), root.name);
        assert_eq!(3, children.len());

        let first_son = &children[0];
        assert_eq!("Cube".to_string(), first_son.name);

        let second_son = &children[1];
        assert_eq!("Lamp".to_string(), second_son.name);

        assert_eq!(0, root.meshes.len());

        assert!(root.metadata.is_none());

        assert_eq!(1.0, root.transformation.a1);
        assert_eq!(1.0, root.transformation.b2);
        assert_eq!(1.0, root.transformation.c3);
        assert_eq!(1.0, root.transformation.d4);
    }

    #[test]
    fn childs_parent_name_matches() {
        use crate::scene::{PostProcess, Scene};

        let current_directory_buf = utils::get_model("models/BLEND/box.blend");

        let scene = Scene::from_file(
            current_directory_buf.as_str(),
            vec![
                PostProcess::CalculateTangentSpace,
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
            ],
        )
        .unwrap();

        let root = scene.root.as_ref().unwrap();
        let children = &root.children;

        let first_son = &children[0];
        let dad = first_son.parent.upgrade().unwrap();

        assert_eq!(root.name, dad.name);
    }

    #[test]
    fn debug_root() {
        use crate::scene::{PostProcess, Scene};

        let current_directory_buf = utils::get_model("models/BLEND/box.blend");

        let scene = Scene::from_file(
            current_directory_buf.as_str(),
            vec![
                PostProcess::CalculateTangentSpace,
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
            ],
        )
        .unwrap();

        dbg!(&scene.root);
    }
}
