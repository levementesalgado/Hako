#!/usr/bin/env python3
"""
Exhaustive search for non-trivial Collatz cycles via the algebraic cycle equation.

(2^V - 3^k) * n = C

where V = Sum v_i, C = Sum_{j=0}^{k-1} 3^{k-1-j} * 2^{Sum_{i=1}^{j} v_i}

For a non-trivial cycle: n > 1, odd, positive.
Each v_i >= 1.

We search all v-sequences with total V up to 30.
"""

import sys
import math
from collections import defaultdict

# Known cycles
KNOWN_CYCLES = {
    1: "trivial 1->1->1->...",
    -1: "negative cycle -1->-1->-1->...",
    -5: "negative cycle -5->-7->-5->...",
}


def compute_C_and_V(v_seq):
    """Compute C and total V from a v-sequence."""
    k = len(v_seq)
    V = sum(v_seq)
    
    C = 0
    for j in range(k):
        pow3_term = 3 ** (k - 1 - j)
        prefix_v = sum(v_seq[:j])  # Sum_{i=1}^{j} v_i (empty sum = 0 for j=0)
        pow2_term = 2 ** prefix_v
        C += pow3_term * pow2_term
    
    return C, V


def compute_n_from_v_seq(v_seq):
    """
    Given a v-sequence (list of v_i, each >= 1),
    compute n = C / (2^V - 3^k).
    Returns (n, V, C, denom) or None if invalid.
    """
    k = len(v_seq)
    C, V = compute_C_and_V(v_seq)
    
    pow2_V = 2 ** V
    pow3_k = 3 ** k
    
    if pow2_V <= pow3_k:
        return None  # denominator must be positive for positive n
    
    denom = pow2_V - pow3_k
    
    if C % denom != 0:
        return None  # n must be integer
    
    n = C // denom
    
    if n <= 0:
        return None
    
    return n, V, C, denom


def trailing_zeros(x):
    """Compute trailing zeros (v2) of a positive integer."""
    if x == 0:
        return 0
    return (x & -x).bit_length() - 1


def collatz_step_condensed(n):
    """
    One step of the condensed Collatz map.
    Returns (next_n, v)
    """
    if n % 2 == 0:
        v = trailing_zeros(n)
        return (n >> v, v)
    else:
        val = 3 * n + 1
        v = trailing_zeros(val)
        return (val >> v, v)


def verify_cycle(n_start, max_steps=100000):
    """
    Verify if n_start is part of a cycle.
    Returns (is_cycle, cycle_length, cycle_start_val, cycle_vals).
    """
    seen = {}
    n = n_start
    steps = 0
    
    while steps < max_steps:
        if n in seen:
            cycle_start_idx = seen[n]
            cycle_length = steps - cycle_start_idx
            return (True, cycle_length, n)
        
        seen[n] = steps
        
        if n == 1:
            return (True, 0, 1)
        
        n, _ = collatz_step_condensed(n)
        steps += 1
        
        if n > 10**100:
            break
    
    return (False, 0, n_start)


def check_congruence_constraints(v_seq, n):
    """
    Check the congruence constraints.
    """
    k = len(v_seq)
    C, V = compute_C_and_V(v_seq)
    pow2_V = 2 ** V
    pow3_k = 3 ** k
    
    if (pow2_V - C) % pow3_k != 0:
        return False
    
    if C >= pow2_V:
        return False
    
    return True


def verify_v_sequence_consistency(v_seq, n):
    """
    Verify that a v-sequence matches the actual Collatz iteration from n.
    Returns (ok, msg).
    """
    x = n
    for i, v_expected in enumerate(v_seq):
        if x % 2 == 0:
            return False, f"Step {i}: n={x} is even"
        
        val = 3 * x + 1
        v_actual = trailing_zeros(val)
        
        if v_actual != v_expected:
            return False, f"Step {i}: n={x}, expected v={v_expected}, actual v={v_actual}"
        
        x = val >> v_actual
        
        if i < len(v_seq) - 1 and x <= 1:
            return False, f"Step {i}: converged too early"
    
    return True, f"Consistent, final={x}"


def enumerate_compositions(k, total_sum, min_val=1):
    """
    Enumerate all compositions of total_sum into k parts, each >= min_val.
    Yields lists of length k.
    """
    if k == 0:
        if total_sum == 0:
            yield []
        return
    
    # Each part at least min_val
    remaining_min = (k - 1) * min_val
    max_first = total_sum - remaining_min
    if max_first < min_val:
        return
    
    for first in range(min_val, max_first + 1):
        for rest in enumerate_compositions(k - 1, total_sum - first, min_val):
            yield [first] + rest


def brute_force_small_cycle_check(limit=100000):
    """Check all odd n up to limit for cycle membership."""
    visited = {}
    total_cycles = []
    
    for n_start in range(3, limit + 1, 2):
        if n_start in visited:
            continue
        
        path = {}
        n = n_start
        is_cycle = False
        cycle_start = None
        
        while n not in visited and n != 1:
            path[n] = len(path)
            visited[n] = 0
            n, _ = collatz_step_condensed(n)
            
            if n in path:
                # We found a cycle within this path
                is_cycle = True
                cycle_start = n
                break
        
        if is_cycle and cycle_start and cycle_start != 1:
            # Trace the cycle
            cycle_vals = []
            x = cycle_start
            while True:
                cycle_vals.append(x)
                x, _ = collatz_step_condensed(x)
                if x == cycle_start:
                    break
            total_cycles.append((n_start, cycle_start, cycle_vals))
            
            # Mark all cycle values
            for cv in cycle_vals:
                visited[cv] = -1  # mark as cycle
        
        if n == 1:
            for p in path:
                visited[p] = 1  # mark as converging to 1
    
    return total_cycles


def check_v_options_by_mod8(v_seq, n):
    """
    Check that each v_i is consistent with the mod 8 residue constraint:
    n_{i-1} mod 4 = 3 => v_i = 1
    n_{i-1} mod 8 = 1 => v_i = 2
    n_{i-1} mod 8 = 5 => v_i >= 3
    """
    k = len(v_seq)
    x = n
    for i, v in enumerate(v_seq):
        r = x % 8
        if r == 3 or r == 7:
            expected = 1
        elif r == 1:
            expected = 2
        elif r == 5:
            expected = -1  # v >= 3, any value >= 3 is ok
        else:
            return False, f"n={x} mod 8 = {r} should not happen for odd n in cycle"
        
        if expected > 0 and v != expected:
            return False, f"Step {i}: n mod 8 = {r} expects v={expected}, got v={v}"
        
        if r == 5 and v < 3:
            return False, f"Step {i}: n mod 8 = 5 expects v>=3, got v={v}"
        
        # Advance
        x = (3 * x + 1) >> v
    
    return True, f"Mod 8 constraints satisfied for v={v_seq}, n={n}"


def main():
    MAX_V = 30
    
    print("=" * 80)
    print("COLLATZ CYCLE SEARCH - Algebraic Cycle Equation")
    print(f"Maximum total V = {MAX_V}")
    print("=" * 80)
    print()
    
    # Phase 0: Brute-force check
    print("Phase 0: Brute-force cycle check for odd n up to 100000...")
    cycles = brute_force_small_cycle_check(100000)
    if cycles:
        print(f"  >>> Found cycles! {len(cycles)} cycle entries:")
        for entry, cycle_start, cycle_vals in cycles[:5]:
            print(f"      n_start={entry}, cycle at {cycle_start}, values={cycle_vals}")
    else:
        print("  No non-trivial cycles found up to n=100000 (confirmed)")
    print()
    
    # Phase 1: Enumerate v-sequences
    print(f"Phase 1: Enumerate v-sequences with V <= {MAX_V}...")
    
    all_candidates = []
    
    # k from 1 to MAX_V
    for k in range(1, MAX_V + 1):
        # Minimum V for 2^V > 3^k
        min_V = math.floor(k * math.log2(3)) + 1
        if min_V > MAX_V:
            continue
        
        for V in range(min_V, MAX_V + 1):
            if 2**V <= 3**k:
                continue
            
            count_for_this_V = 0
            for v_seq in enumerate_compositions(k, V, 1):
                result = compute_n_from_v_seq(v_seq)
                if result is not None:
                    n, Vc, C, denom = result
                    if n > 1 and n % 2 == 1:
                        all_candidates.append((v_seq, n, C, denom))
                        count_for_this_V += 1
            
            if count_for_this_V > 0:
                pass  # Progress reporting would go here
    
    print(f"  Total candidates (n odd integer > 1): {len(all_candidates)}")
    print()
    
    # Phase 2: Check congruence and consistency
    print("Phase 2: Checking congruence constraints and consistency...")
    
    valid_candidates = []
    
    for v_seq, n, C, denom in all_candidates:
        k = len(v_seq)
        V = sum(v_seq)
        
        # Skip known cycles
        if n in KNOWN_CYCLES:
            continue
        
        # Check congruence
        if not check_congruence_constraints(v_seq, n):
            continue
        
        # Check mod 8 consistency
        ok_mod8, msg_mod8 = check_v_options_by_mod8(v_seq, n)
        if not ok_mod8:
            continue
        
        # Verify v-sequence consistency with actual trajectory
        ok_consist, msg_consist = verify_v_sequence_consistency(v_seq, n)
        if not ok_consist:
            continue
        
        # Check if it's actually a cycle
        is_cycle, cycle_len, cycle_start = verify_cycle(n)
        
        if is_cycle and cycle_len > 0 and n != 1:
            valid_candidates.append((v_seq, n, C, denom, k, V, cycle_len))
        elif is_cycle and cycle_len == 0:
            # Converges to 1
            pass
    
    # Phase 3: Report
    print()
    if valid_candidates:
        print(f"  >>> VALID NON-TRIVIAL CYCLES FOUND: {len(valid_candidates)}")
        for v_seq, n, C, denom, k, V, cycle_len in valid_candidates:
            print(f"      k={k}, V={V}, v={v_seq}")
            print(f"      n={n}, C={C}, denom={denom}")
            print(f"      Cycle length (condensed steps): {cycle_len}")
            print()
    else:
        print("  No non-trivial cycles found.")
    
    # Summary analysis for most promising candidates
    print()
    print("Phase 3: Analysis of closest misses...")
    
    # Find candidates where n is odd, integer, but fails consistency
    inconsistencies = []
    for v_seq, n, C, denom in all_candidates:
        k = len(v_seq)
        V = sum(v_seq)
        if n in KNOWN_CYCLES:
            continue
        if not check_congruence_constraints(v_seq, n):
            continue
        ok_consist, msg = verify_v_sequence_consistency(v_seq, n)
        if not ok_consist:
            inconsistencies.append((v_seq, n, C, denom, k, V, msg))
    
    inconsistencies.sort(key=lambda x: x[4])  # sort by k
    
    if inconsistencies:
        print(f"  Found {len(inconsistencies)} candidates that satisfy equation but fail bit consistency.")
        print("  Top 20 by k:")
        for v_seq, n, C, denom, k, V, msg in inconsistencies[:20]:
            print(f"    k={k:3d}, V={V:3d}, n={n:>20}, v={v_seq}")
            print(f"      Reason: {msg}")
    
    print()
    print("=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print(f"  Search bound: total sum(v_i) <= {MAX_V}")
    print(f"  Total candidates (n odd integer > 1): {len(all_candidates)}")
    print(f"  Passed congruence + mod8 + consistency: {len(valid_candidates)}")
    print(f"  Verified non-trivial cycles: {len(valid_candidates)}")
    print()
    
    if len(valid_candidates) == 0:
        print("CONCLUSION: No non-trivial Collatz cycles exist up to V <= 30.")
        print("This is consistent with the Collatz conjecture.")
    else:
        print("CONCLUSION: New non-trivial cycles found!")
        for v_seq, n, C, denom, k, V, cycle_len in valid_candidates:
            print(f"  Cycle: n={n}, k={k}, V={V}")
    
    return valid_candidates


if __name__ == "__main__":
    main()
