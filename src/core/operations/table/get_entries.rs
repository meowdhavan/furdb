use crate::core::utils;

use bitvec::prelude::*;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::os::unix::fs::FileExt;

use crate::core::models::EntriesResult;
use crate::core::models::Entry;
use crate::core::models::Table;

use crate::core::error::EntryReadError;

impl Table {
    pub fn query(&self, column_index: u64, value: u128) -> Result<EntriesResult, EntryReadError> {
        let config = self.get_config();
        let table_info = self.get_table_info();

        let table_columns = table_info.get_table_columns();

        if column_index >= table_columns.len() as u64 {
            return Err(EntryReadError::InvalidColumn);
        }

        let entry_size = table_columns
            .iter()
            .fold(0, |acc, column| acc + column.get_size()) as u64
            / 8;

        let table_data_path = utils::get_table_data_path(
            &config.workdir,
            &table_info.get_database_id(),
            &table_info.get_table_id(),
        );

        let table_data_file = std::fs::OpenOptions::new()
            .read(true)
            .open(table_data_path)
            .map_err(|e| EntryReadError::OtherError(e.to_string()))?;

        let table_data_file_size = table_data_file
            .metadata()
            .map_err(|e| EntryReadError::OtherError(e.to_string()))?
            .len();

        let entry_count = table_data_file_size / entry_size;

        let identifier_size = if entry_count > 0 {
            1 + ((entry_count - 1) / 256)
        } else {
            0
        };

        let table_sortfile_path = utils::get_sortfile_path(
            &config.workdir,
            &table_info.get_database_id(),
            &table_info.get_table_id(),
        );

        let table_sortfile = std::fs::OpenOptions::new()
            .read(true)
            .open(table_sortfile_path)
            .map_err(|e| EntryReadError::OtherError(e.to_string()))?;

        // Lower Bound

        let mut left = (column_index * entry_count) as i128;
        // The slice of the sortfile belonging to this column ends at
        // `(column_index + 1) * entry_count - 1`. Distributing the `- 1`
        // over the product instead would put the bound `column_index`
        // entries short, hiding the column's largest values.
        let mut right = ((column_index + 1) as i128) * (entry_count as i128) - 1;

        while left <= right {
            let mid = left + (right - left) / 2;

            let mut index_buf = vec![0u8; identifier_size as usize];
            table_sortfile
                .read_exact_at(&mut index_buf, mid as u64 * identifier_size)
                .map_err(|e| EntryReadError::OtherError(e.to_string()))?;
            let index_bin = BitVec::<u8, Msb0>::from_slice(&index_buf);

            let index = index_bin
                .into_iter()
                .fold(0, |acc, bit| (acc << 1) + (bit as u64));

            let current_value = self.get_entry(index)?.get_data()[column_index as usize];

            if current_value >= value {
                right = mid - 1;
            } else {
                left = mid + 1;
            }
        }

        let start = left;

        // Upper Bound

        let mut left = (column_index * entry_count) as i128;
        // The slice of the sortfile belonging to this column ends at
        // `(column_index + 1) * entry_count - 1`. Distributing the `- 1`
        // over the product instead would put the bound `column_index`
        // entries short, hiding the column's largest values.
        let mut right = ((column_index + 1) as i128) * (entry_count as i128) - 1;

        while left <= right {
            let mid = left + (right - left) / 2;

            let mut index_buf = vec![0u8; identifier_size as usize];
            table_sortfile
                .read_exact_at(&mut index_buf, mid as u64 * identifier_size)
                .map_err(|e| EntryReadError::OtherError(e.to_string()))?;
            let index_bin = BitVec::<u8, Msb0>::from_slice(&index_buf);

            let index = index_bin
                .into_iter()
                .fold(0, |acc, bit| (acc << 1) + (bit as u64));

            let current_value = self.get_entry(index)?.get_data()[column_index as usize];

            if current_value <= value {
                left = mid + 1;
            } else {
                right = mid - 1;
            }
        }

        let end = left;

        let indices = (start..end)
            .map(|offset| {
                let mut index_bin = vec![0u8; identifier_size as usize];
                table_sortfile
                    .read_exact_at(&mut index_bin, offset as u64 * identifier_size)
                    .unwrap();
                let index_bin = BitVec::<u8, Msb0>::from_slice(&index_bin);

                index_bin
                    .into_iter()
                    .fold(0, |acc, bit| (acc << 1) + (bit as u64))
            })
            .collect();

        self.get_entries(indices)
    }

    pub fn get_all_entries(&self) -> Result<EntriesResult, EntryReadError> {
        let config = self.get_config();
        let table_info = self.get_table_info();

        let table_data_path = utils::get_table_data_path(
            &config.workdir,
            &table_info.get_database_id(),
            &table_info.get_table_id(),
        );

        let table_data_file = std::fs::OpenOptions::new()
            .read(true)
            .open(table_data_path)
            .unwrap();

        let entry_size = table_info
            .get_table_columns()
            .iter()
            .fold(0, |acc, column| acc + column.get_size()) as u64
            / 8;

        let file_size = table_data_file
            .metadata()
            .map_err(|e| EntryReadError::OtherError(e.to_string()))?
            .len();

        let indices = (0..file_size / entry_size).collect();

        self.get_entries(indices)
    }

    pub fn get_entries(&self, indices: Vec<u64>) -> Result<EntriesResult, EntryReadError> {
        let config = self.get_config();
        let table_info = self.get_table_info();

        let table_data_path = utils::get_table_data_path(
            &config.workdir,
            &table_info.get_database_id(),
            &table_info.get_table_id(),
        );

        let table_data_file = std::fs::OpenOptions::new()
            .read(true)
            .open(table_data_path)
            .unwrap();

        let entry_size = table_info
            .get_table_columns()
            .iter()
            .fold(0, |acc, column| acc + column.get_size()) as u64
            / 8;

        let file_size = table_data_file
            .metadata()
            .map_err(|e| EntryReadError::OtherError(e.to_string()))?
            .len();

        // TODO: Refactor this in the future

        for index in &indices {
            if *index >= file_size / entry_size {
                return Err(EntryReadError::InvalidIndex);
            }
        }

        let result = EntriesResult::new(
            &indices
                .into_iter()
                .map(|index| self.get_entry(index).unwrap())
                .collect::<Vec<Entry>>(),
        );

        Ok(result)
    }

    pub fn get_entry(&self, index: u64) -> Result<Entry, EntryReadError> {
        let config = self.get_config();
        let table_info = self.get_table_info();

        let table_columns = table_info.get_table_columns();

        let entry_size = table_columns
            .iter()
            .fold(0, |acc, column| acc + column.get_size()) as u64
            / 8;

        let table_data_path = utils::get_table_data_path(
            &config.workdir,
            &table_info.get_database_id(),
            &table_info.get_table_id(),
        );

        let mut table_data_file = std::fs::OpenOptions::new()
            .read(true)
            .open(table_data_path)
            .map_err(|e| EntryReadError::OtherError(e.to_string()))?;

        let table_data_file_size = table_data_file
            .metadata()
            .map_err(|e| EntryReadError::OtherError(e.to_string()))?
            .len();

        let entry_count = table_data_file_size / entry_size;

        if index >= entry_count {
            return Err(EntryReadError::InvalidIndex);
        }

        table_data_file
            .seek(SeekFrom::Start(index * entry_size))
            .map_err(|e| match e.kind() {
                ErrorKind::InvalidInput => EntryReadError::InvalidIndex,
                _ => EntryReadError::OtherError(e.to_string()),
            })?;

        let mut buf = vec![0u8; entry_size as usize];

        table_data_file
            .read_exact(&mut buf)
            .map_err(|e| EntryReadError::OtherError(e.to_string()))?;

        let entry_bin: BitVec<u8, Msb0> = BitVec::from_slice(&buf);

        let result = table_columns
            .iter()
            .fold((Vec::new(), 0), |(mut acc, column_start), column| {
                let column_size = column.get_size() as usize;

                let data_bin = &entry_bin[column_start..(column_start + column_size)];
                let data_bin = BitVec::from(data_bin);
                let column_start = column_start + column_size;

                let data = data_bin
                    .into_iter()
                    .fold(0, |acc, bit| (acc << 1) + (bit as u128));

                acc.push(data);
                (acc, column_start)
            })
            .0;

        Ok(Entry::new(index as usize, result))
    }
}
