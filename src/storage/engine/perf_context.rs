// Copyright 2018 PingCAP, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.

use std::ops::{Deref, DerefMut};

use rocksdb::PerfContext;

#[derive(Default, Debug, Clone, Copy, Add, AddAssign, Sub, SubAssign)]
pub struct PerfStatisticsFields {
    pub user_key_comparison_count: usize,
    pub block_cache_hit_count: usize,
    pub block_read_count: usize,
    pub block_read_byte: usize,
    pub block_read_time: usize,
    pub block_checksum_time: usize,
    pub block_decompress_time: usize,
    pub get_read_bytes: usize,
    pub multiget_read_bytes: usize,
    pub iter_read_bytes: usize,
    pub internal_key_skipped_count: usize,
    pub internal_delete_skipped_count: usize,
    pub internal_recent_skipped_count: usize,
    pub internal_merge_count: usize,
    pub get_snapshot_time: usize,
    pub get_from_memtable_time: usize,
    pub get_from_memtable_count: usize,
    pub get_post_process_time: usize,
    pub get_from_output_files_time: usize,
    pub seek_on_memtable_time: usize,
    pub seek_on_memtable_count: usize,
    pub next_on_memtable_count: usize,
    pub prev_on_memtable_count: usize,
    pub seek_child_seek_time: usize,
    pub seek_child_seek_count: usize,
    pub seek_min_heap_time: usize,
    pub seek_max_heap_time: usize,
    pub seek_internal_seek_time: usize,
    pub find_next_user_entry_time: usize,
    pub write_wal_time: usize,
    pub write_memtable_time: usize,
    pub write_delay_time: usize,
    pub write_pre_and_post_process_time: usize,
    pub db_mutex_lock_nanos: usize,
    pub db_condition_wait_nanos: usize,
    pub merge_operator_time_nanos: usize,
    pub read_index_block_nanos: usize,
    pub read_filter_block_nanos: usize,
    pub new_table_block_iter_nanos: usize,
    pub new_table_iterator_nanos: usize,
    pub block_seek_nanos: usize,
    pub find_table_nanos: usize,
    pub bloom_memtable_hit_count: usize,
    pub bloom_memtable_miss_count: usize,
    pub bloom_sst_hit_count: usize,
    pub bloom_sst_miss_count: usize,
    pub env_new_sequential_file_nanos: usize,
    pub env_new_random_access_file_nanos: usize,
    pub env_new_writable_file_nanos: usize,
    pub env_reuse_writable_file_nanos: usize,
    pub env_new_random_rw_file_nanos: usize,
    pub env_new_directory_nanos: usize,
    pub env_file_exists_nanos: usize,
    pub env_get_children_nanos: usize,
    pub env_get_children_file_attributes_nanos: usize,
    pub env_delete_file_nanos: usize,
    pub env_create_dir_nanos: usize,
    pub env_create_dir_if_missing_nanos: usize,
    pub env_delete_dir_nanos: usize,
    pub env_get_file_size_nanos: usize,
    pub env_get_file_modification_time_nanos: usize,
    pub env_rename_file_nanos: usize,
    pub env_link_file_nanos: usize,
    pub env_lock_file_nanos: usize,
    pub env_unlock_file_nanos: usize,
    pub env_new_logger_nanos: usize,
}

/// Store statistics we need. Data comes from RocksDB's `PerfContext`.
/// This statistics store instant values.
#[derive(Debug, Clone, Copy)]
pub struct PerfStatisticsInstant(pub PerfStatisticsFields);

impl PerfStatisticsInstant {
    /// Create an instance which stores instant statistics values, retrieved at creation.
    pub fn new() -> Self {
        let perf_context = PerfContext::get();
        PerfStatisticsInstant(PerfStatisticsFields {
            user_key_comparison_count: perf_context.user_key_comparison_count() as usize,
            block_cache_hit_count: perf_context.block_cache_hit_count() as usize,
            block_read_count: perf_context.block_read_count() as usize,
            block_read_byte: perf_context.block_read_byte() as usize,
            block_read_time: perf_context.block_read_time() as usize,
            block_checksum_time: perf_context.block_checksum_time() as usize,
            block_decompress_time: perf_context.block_decompress_time() as usize,
            get_read_bytes: perf_context.get_read_bytes() as usize,
            multiget_read_bytes: perf_context.multiget_read_bytes() as usize,
            iter_read_bytes: perf_context.iter_read_bytes() as usize,
            internal_key_skipped_count: perf_context.internal_key_skipped_count() as usize,
            internal_delete_skipped_count: perf_context.internal_delete_skipped_count() as usize,
            internal_recent_skipped_count: perf_context.internal_recent_skipped_count() as usize,
            internal_merge_count: perf_context.internal_merge_count() as usize,
            get_snapshot_time: perf_context.get_snapshot_time() as usize,
            get_from_memtable_time: perf_context.get_from_memtable_time() as usize,
            get_from_memtable_count: perf_context.get_from_memtable_count() as usize,
            get_post_process_time: perf_context.get_post_process_time() as usize,
            get_from_output_files_time: perf_context.get_from_output_files_time() as usize,
            seek_on_memtable_time: perf_context.seek_on_memtable_time() as usize,
            seek_on_memtable_count: perf_context.seek_on_memtable_count() as usize,
            next_on_memtable_count: perf_context.next_on_memtable_count() as usize,
            prev_on_memtable_count: perf_context.prev_on_memtable_count() as usize,
            seek_child_seek_time: perf_context.seek_child_seek_time() as usize,
            seek_child_seek_count: perf_context.seek_child_seek_count() as usize,
            seek_min_heap_time: perf_context.seek_min_heap_time() as usize,
            seek_max_heap_time: perf_context.seek_max_heap_time() as usize,
            seek_internal_seek_time: perf_context.seek_internal_seek_time() as usize,
            find_next_user_entry_time: perf_context.find_next_user_entry_time() as usize,
            write_wal_time: perf_context.write_wal_time() as usize,
            write_memtable_time: perf_context.write_memtable_time() as usize,
            write_delay_time: perf_context.write_delay_time() as usize,
            write_pre_and_post_process_time: perf_context.write_pre_and_post_process_time()
                as usize,
            db_mutex_lock_nanos: perf_context.db_mutex_lock_nanos() as usize,
            db_condition_wait_nanos: perf_context.db_condition_wait_nanos() as usize,
            merge_operator_time_nanos: perf_context.merge_operator_time_nanos() as usize,
            read_index_block_nanos: perf_context.read_index_block_nanos() as usize,
            read_filter_block_nanos: perf_context.read_filter_block_nanos() as usize,
            new_table_block_iter_nanos: perf_context.new_table_block_iter_nanos() as usize,
            new_table_iterator_nanos: perf_context.new_table_iterator_nanos() as usize,
            block_seek_nanos: perf_context.block_seek_nanos() as usize,
            find_table_nanos: perf_context.find_table_nanos() as usize,
            bloom_memtable_hit_count: perf_context.bloom_memtable_hit_count() as usize,
            bloom_memtable_miss_count: perf_context.bloom_memtable_miss_count() as usize,
            bloom_sst_hit_count: perf_context.bloom_sst_hit_count() as usize,
            bloom_sst_miss_count: perf_context.bloom_sst_miss_count() as usize,
            env_new_sequential_file_nanos: perf_context.env_new_sequential_file_nanos() as usize,
            env_new_random_access_file_nanos: perf_context.env_new_random_access_file_nanos()
                as usize,
            env_new_writable_file_nanos: perf_context.env_new_writable_file_nanos() as usize,
            env_reuse_writable_file_nanos: perf_context.env_reuse_writable_file_nanos() as usize,
            env_new_random_rw_file_nanos: perf_context.env_new_random_rw_file_nanos() as usize,
            env_new_directory_nanos: perf_context.env_new_directory_nanos() as usize,
            env_file_exists_nanos: perf_context.env_file_exists_nanos() as usize,
            env_get_children_nanos: perf_context.env_get_children_nanos() as usize,
            env_get_children_file_attributes_nanos: perf_context
                .env_get_children_file_attributes_nanos()
                as usize,
            env_delete_file_nanos: perf_context.env_delete_file_nanos() as usize,
            env_create_dir_nanos: perf_context.env_create_dir_nanos() as usize,
            env_create_dir_if_missing_nanos: perf_context.env_create_dir_if_missing_nanos()
                as usize,
            env_delete_dir_nanos: perf_context.env_delete_dir_nanos() as usize,
            env_get_file_size_nanos: perf_context.env_get_file_size_nanos() as usize,
            env_get_file_modification_time_nanos: perf_context
                .env_get_file_modification_time_nanos()
                as usize,
            env_rename_file_nanos: perf_context.env_rename_file_nanos() as usize,
            env_link_file_nanos: perf_context.env_link_file_nanos() as usize,
            env_lock_file_nanos: perf_context.env_lock_file_nanos() as usize,
            env_unlock_file_nanos: perf_context.env_unlock_file_nanos() as usize,
            env_new_logger_nanos: perf_context.env_new_logger_nanos() as usize,
        })
    }

    /// Calculate delta values until now.
    pub fn delta(&self) -> PerfStatisticsDelta {
        let now = Self::new();
        PerfStatisticsDelta(now.0 - self.0)
    }
}

impl Deref for PerfStatisticsInstant {
    type Target = PerfStatisticsFields;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PerfStatisticsInstant {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Store statistics we need. Data comes from RocksDB's `PerfContext`.
/// This this statistics store delta values between two instant statistics.
#[derive(Default, Debug, Clone, Copy, Add, AddAssign, Sub, SubAssign)]
pub struct PerfStatisticsDelta(pub PerfStatisticsFields);

impl Deref for PerfStatisticsDelta {
    type Target = PerfStatisticsFields;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PerfStatisticsDelta {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_operations() {
        let f1 = PerfStatisticsFields {
            internal_key_skipped_count: 1,
            internal_delete_skipped_count: 2,
            block_cache_hit_count: 3,
            block_read_count: 4,
            block_read_byte: 5,
        };
        let f2 = PerfStatisticsFields {
            internal_key_skipped_count: 2,
            internal_delete_skipped_count: 3,
            block_cache_hit_count: 5,
            block_read_count: 7,
            block_read_byte: 11,
        };
        let f3 = f1 + f2;
        assert_eq!(f3.internal_key_skipped_count, 3);
        assert_eq!(f3.block_cache_hit_count, 8);
        assert_eq!(f3.block_read_byte, 16);

        let mut f3 = f1;
        f3 += f2;
        assert_eq!(f3.internal_key_skipped_count, 3);
        assert_eq!(f3.block_cache_hit_count, 8);
        assert_eq!(f3.block_read_byte, 16);

        let f3 = f2 - f1;
        assert_eq!(f3.internal_key_skipped_count, 1);
        assert_eq!(f3.block_cache_hit_count, 2);
        assert_eq!(f3.block_read_byte, 6);

        let mut f3 = f2;
        f3 -= f1;
        assert_eq!(f3.internal_key_skipped_count, 1);
        assert_eq!(f3.block_cache_hit_count, 2);
        assert_eq!(f3.block_read_byte, 6);
    }

    #[test]
    fn test_deref() {
        let mut stats = PerfStatisticsDelta(PerfStatisticsFields {
            internal_key_skipped_count: 1,
            internal_delete_skipped_count: 2,
            block_cache_hit_count: 3,
            block_read_count: 4,
            block_read_byte: 5,
        });
        assert_eq!(stats.block_cache_hit_count, 3);
        stats.block_cache_hit_count = 6;
        assert_eq!(stats.block_cache_hit_count, 6);
    }
}
*/
