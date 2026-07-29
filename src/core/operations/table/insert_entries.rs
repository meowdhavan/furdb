use crate::core::utils;

use crate::core::models::Table;

use bitvec::prelude::*;
use std::io::Write;

use crate::core::error::EntryInsertionError;

impl Table {
    pub fn insert_entries(&self, entries: &[Vec<u128>]) -> Result<(), EntryInsertionError> {
        let config = self.get_config();
        let table_info = self.get_table_info();

        let mut data = BitVec::<u8, Msb0>::new();

        let table_columns = table_info.get_table_columns();

        for entry in entries {
            if entry.len() != table_columns.len() {
                return Err(EntryInsertionError::ColumnMismatch);
            }

            for index in 0..table_columns.len() {
                let element_size = table_columns[index].get_size() as usize;
                let mut element = entry[index];

                let mut current_entry_bin = BitVec::<u8, Msb0>::new();

                // `2u128.pow(element_size)` overflows once a column is 128 bits
                // wide, so the bound is expressed as a shift that is only taken
                // when it fits — a 128-bit column holds any `u128`.
                if element_size < 128 && element >= (1u128 << element_size) {
                    return Err(EntryInsertionError::ColumnOverflow);
                }

                while element > 0 {
                    current_entry_bin.push(element % 2 == 1);
                    element /= 2;
                }

                while current_entry_bin.len() < element_size {
                    current_entry_bin.push(false);
                }

                current_entry_bin.reverse();

                data.append(&mut current_entry_bin);
            }
        }

        let table_data_path = utils::get_table_data_path(
            &config.workdir,
            &table_info.get_database_id(),
            &table_info.get_table_id(),
        );

        let mut table_data_file = std::fs::OpenOptions::new()
            .append(true)
            .open(table_data_path)
            .map_err(|e| EntryInsertionError::OtherError(e.to_string()))?;

        table_data_file
            .write(&Vec::<u8>::from(data))
            .map_err(|e| EntryInsertionError::OtherError(e.to_string()))?;

        self.generate_sortfile()
            .map_err(|e| EntryInsertionError::OtherError(e.to_string()))?;

        Ok(())
    }
}
