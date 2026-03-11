use crate::static_store::data::{StaticBasic, StaticMap, StaticObject, StaticStruct, StaticTable};
use crate::store::TreePrint;
use crate::store::data::{ContainerDefinition, ContainerItem};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StaticProperty {
    Basic(StaticBasic),
    Table(StaticTable),
    Object(StaticObject),
    Struct(StaticStruct),
    Map(StaticMap),
}

impl From<ContainerItem> for StaticProperty {
    fn from(item: ContainerItem) -> Self {
        match item {
            ContainerItem::Basic(b) => Self::Basic(StaticBasic::from(&b)),
            ContainerItem::Table(t) => Self::Table(StaticTable::from(&t)),
            ContainerItem::Container(c) => match c.definition() {
                ContainerDefinition::Object(_) => Self::Object(StaticObject::from(&c)),
                ContainerDefinition::Struct(_) => Self::Struct(StaticStruct::from(&c)),
                ContainerDefinition::Map(_) => Self::Map(StaticMap::from(&c)),
            },
        }
    }
}

impl StaticProperty {
    pub fn hash(&self) -> [u8; 32] {
        match self {
            Self::Basic(b) => b.hash(),
            Self::Table(t) => t.hash(),
            Self::Object(o) => o.hash(),
            Self::Struct(s) => s.hash(),
            Self::Map(m) => m.hash(),
        }
    }

    pub fn get_basic(&self) -> Option<&StaticBasic> {
        match self {
            Self::Basic(b) => Some(b),
            _ => None,
        }
    }

    pub fn get_table(&self) -> Option<&StaticTable> {
        match self {
            Self::Table(t) => Some(t),
            _ => None,
        }
    }

    pub fn get_object(&self) -> Option<&StaticObject> {
        match self {
            Self::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn get_struct(&self) -> Option<&StaticStruct> {
        match self {
            Self::Struct(s) => Some(s),
            _ => None,
        }
    }

    pub fn get_map(&self) -> Option<&StaticMap> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }
}

impl TreePrint for StaticProperty {
    fn tree_print(&self, label: &str, prefix: &str, last: bool) {
        match self {
            Self::Basic(b) => b.tree_print(label, prefix, last),
            Self::Table(t) => t.tree_print(label, prefix, last),
            Self::Object(o) => o.tree_print(label, prefix, last),
            Self::Struct(s) => s.tree_print(label, prefix, last),
            Self::Map(m) => m.tree_print(label, prefix, last),
        }
    }
}
