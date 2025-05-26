use core::ops::{Index, IndexMut};

use crate::keyboard::action::KeyAction;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LayersError {
    HighestLayerReached,
    LowestLayerReached,
}

#[derive(Debug, Copy, Clone)]
pub struct Layer<const M: usize, const N: usize> {
    keys: [[KeyAction; N]; M],
}

impl<const M: usize, const N: usize> Layer<M, N> {
    pub const fn new(keys: [[KeyAction; N]; M]) -> Self {
        Self { keys }
    }
}

impl<const M: usize, const N: usize> Default for Layer<M, N> {
    fn default() -> Self {
        Self {
            keys: [[KeyAction::Transparent; N]; M],
        }
    }
}

impl<const M: usize, const N: usize> Index<(usize, usize)> for Layer<M, N> {
    type Output = KeyAction;
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        &self.keys[row][col]
    }
}

impl<const M: usize, const N: usize> IndexMut<(usize, usize)> for Layer<M, N> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        &mut self.keys[row][col]
    }
}

pub struct Layers<const L: usize, const M: usize, const N: usize> {
    layers: [Layer<M, N>; L],
    current_layer: usize,
}

impl<const L: usize, const M: usize, const N: usize> Layers<L, M, N> {
    pub const fn new(layers: [Layer<M, N>; L]) -> Self {
        Self {
            layers,
            current_layer: 0,
        }
    }

    pub fn get_key(&self, row: usize, col: usize) -> KeyAction {
        self.get_key_from_layer(self.current_layer, row, col)
    }

    fn get_key_from_layer(&self, layer: usize, row: usize, col: usize) -> KeyAction {
        match self.layers[layer][(row, col)] {
            KeyAction::Transparent => {
                if layer == 0 {
                    // If we are a the lowest layer, then there is no operation done
                    KeyAction::NoOp
                } else {
                    // Else, get the key in the next layer
                    self.get_key_from_layer(layer - 1, row, col)
                }
            }
            key => key,
        }
    }

    pub fn set_key_from_layer(&mut self, layer: usize, row: usize, col: usize, key: KeyAction) {
        self.layers[layer][(row, col)] = key;
    }

    pub fn layer_up(&mut self) -> Result<(), LayersError> {
        if self.current_layer >= L {
            Err(LayersError::HighestLayerReached)
        } else {
            self.current_layer += 1;
            Ok(())
        }
    }

    pub fn layer_down(&mut self) -> Result<(), LayersError> {
        if self.current_layer == 0 {
            Err(LayersError::LowestLayerReached)
        } else {
            self.current_layer -= 1;
            Ok(())
        }
    }

    pub fn get_current_layer_id(&self) -> usize {
        self.current_layer
    }

    pub fn get_current_layer(&self) -> &Layer<M, N> {
        &self.layers[self.current_layer]
    }

    pub fn get_layer(&self, layer: usize) -> &Layer<M, N> {
        &self.layers[layer]
    }

    pub fn set_current_layer(&mut self, layer: usize) {
        assert!(layer < L, "The current layer cannot b");
        self.current_layer = layer;
    }

    pub fn get_layer_from_key(&self, key: KeyAction) -> Option<usize> {
        if let KeyAction::Layer(layer) = key {
            // Check if the layer is in range
            if layer < L {
                return Some(layer);
            }
        }
        None
    }
}

impl<const L: usize, const M: usize, const N: usize> Default for Layers<L, M, N> {
    fn default() -> Self {
        Self {
            layers: [Layer::default(); L],
            current_layer: 0,
        }
    }
}
