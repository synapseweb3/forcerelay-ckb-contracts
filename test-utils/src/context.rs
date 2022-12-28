use std::collections::HashMap;

use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::{
    bytes::Bytes,
    core::{
        cell::{CellMeta, CellMetaBuilder, ResolvedTransaction},
        Capacity, DepType, HeaderView, TransactionView,
    },
    packed,
    prelude::*,
};

use crate::misc;

#[derive(Default)]
pub struct Context {
    cells: HashMap<packed::OutPoint, (packed::CellOutput, Bytes)>,
    _headers: HashMap<packed::Byte32, HeaderView>,

    cells_by_data_hash: HashMap<packed::Byte32, packed::OutPoint>,
    cells_by_type_hash: HashMap<packed::Byte32, packed::OutPoint>,
}

pub struct DeployedCell {
    out_point: packed::OutPoint,
    cell_output: packed::CellOutput,

    data_hash: packed::Byte32,
    type_hash_opt: Option<packed::Byte32>,
    data: Bytes,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn deploy(
        &mut self,
        data: Bytes,
        lock_script: packed::Script,
        type_script_opt: Option<packed::Script>,
    ) -> DeployedCell {
        let data_hash = packed::CellOutput::calc_data_hash(&data);
        let type_hash_opt = type_script_opt.as_ref().map(|s| s.calc_script_hash());
        let data_capacity = Capacity::bytes(data.len()).unwrap();
        let cell_output = packed::CellOutput::new_builder()
            .lock(lock_script)
            .type_(type_script_opt.pack())
            .build_exact_capacity(data_capacity)
            .unwrap();
        let out_point = misc::random_out_point();

        self.cells
            .insert(out_point.clone(), (cell_output.clone(), data.clone()));
        self.cells_by_data_hash
            .insert(data_hash.clone(), out_point.clone());
        if let Some(ref type_hash) = type_hash_opt {
            self.cells_by_type_hash
                .insert(type_hash.clone(), out_point.clone());
        }

        DeployedCell {
            out_point,
            cell_output,
            data_hash,
            type_hash_opt,
            data,
        }
    }

    pub fn resolve(&self, transaction: TransactionView) -> ResolvedTransaction {
        let resolved_inputs = transaction
            .inputs()
            .into_iter()
            .map(|input| self.build_cell_meta(input.previous_output()))
            .collect();
        let resolved_cell_deps = transaction
            .cell_deps()
            .into_iter()
            .map(|cell_dep| self.build_cell_meta(cell_dep.out_point()))
            .collect();
        ResolvedTransaction {
            transaction,
            resolved_cell_deps,
            resolved_inputs,
            resolved_dep_groups: Default::default(),
        }
    }

    fn build_cell_meta(&self, out_point: packed::OutPoint) -> CellMeta {
        let (cell_output, data) = self.cells.get(&out_point).unwrap();
        CellMetaBuilder::from_cell_output(cell_output.to_owned(), data.to_vec().into())
            .out_point(out_point)
            .build()
    }
}

impl DeployedCell {
    pub fn out_point(&self) -> packed::OutPoint {
        self.out_point.clone()
    }

    pub fn cell_output(&self) -> packed::CellOutput {
        self.cell_output.clone()
    }

    pub fn data_hash(&self) -> packed::Byte32 {
        self.data_hash.clone()
    }

    pub fn type_hash(&self) -> Option<packed::Byte32> {
        self.type_hash_opt.clone()
    }

    pub fn data(&self) -> Bytes {
        self.data.clone()
    }

    pub fn as_cell_dep(&self) -> packed::CellDep {
        packed::CellDep::new_builder()
            .out_point(self.out_point())
            .dep_type(DepType::Code.into())
            .build()
    }

    pub fn as_input(&self) -> packed::CellInput {
        packed::CellInput::new(self.out_point(), 0)
    }
}

impl CellDataProvider for Context {
    fn get_cell_data(&self, out_point: &packed::OutPoint) -> Option<Bytes> {
        self.cells
            .get(out_point)
            .map(|(_, data)| Bytes::from(data.to_vec()))
    }
    fn get_cell_data_hash(&self, out_point: &packed::OutPoint) -> Option<packed::Byte32> {
        self.cells
            .get(out_point)
            .map(|(_, data)| packed::CellOutput::calc_data_hash(data))
    }
}

impl HeaderProvider for Context {
    fn get_header(&self, block_hash: &packed::Byte32) -> Option<HeaderView> {
        self._headers.get(block_hash).cloned()
    }
}
