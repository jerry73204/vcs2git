use color_eyre::{eyre::bail, Result};
use std::{collections::HashSet, fmt::Debug, hash::Hash};

/// Check if a subset is fully contained within a larger set
pub fn check_subset<T>(all: &HashSet<T>, subset: &HashSet<T>) -> Result<()>
where
    T: Eq + Hash + Debug,
{
    if !all.is_superset(subset) {
        let diff: Vec<_> = subset.difference(all).collect();
        bail!("Repositories not found: {diff:?}");
    }

    Ok(())
}

/// Check if two sets are disjoint (have no common elements)
pub fn check_disjoint<T>(lset: &HashSet<T>, rset: &HashSet<T>) -> Result<()>
where
    T: Eq + Hash + Debug,
{
    if !lset.is_disjoint(rset) {
        let inter: Vec<_> = lset.intersection(rset).collect();
        bail!("Repositories cannot be selected and skipped at the same time: {inter:?}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_disjoint_logic() {
        let set1: HashSet<i32> = [1, 2, 3].iter().cloned().collect();
        let set2: HashSet<i32> = [3, 4, 5].iter().cloned().collect();
        let set3: HashSet<i32> = [6, 7, 8].iter().cloned().collect();

        // Should fail because sets are not disjoint (have 3 in common)
        assert!(check_disjoint(&set1, &set2).is_err());

        // Should succeed because sets are disjoint
        assert!(check_disjoint(&set1, &set3).is_ok());
    }

    #[test]
    fn test_check_subset() {
        let all: HashSet<&str> = ["a", "b", "c", "d"].iter().cloned().collect();
        let subset: HashSet<&str> = ["a", "c"].iter().cloned().collect();
        let not_subset: HashSet<&str> = ["a", "e"].iter().cloned().collect();

        // Should succeed because subset is contained in all
        assert!(check_subset(&all, &subset).is_ok());

        // Should fail because not_subset contains "e" which is not in all
        assert!(check_subset(&all, &not_subset).is_err());
    }
}
