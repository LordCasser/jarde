from pathlib import Path
import re
root=Path('/Users/lordcasser/workspace/projects/jarde');raw=(root/'openspec/changes/preserve-proved-return-arm-loop-latch-origins/results/diagnostic-root-v2/region-with-diagnostic.rs').read_text()
start=raw.index('    #[test]\n    fn diagnose_cf07_last_index_latch_region_ownership_from_real_class_ir()')
setup=raw[start:raw.index('        eprintln!("lastIndexOf recovered regions:',start)]
setup=setup.replace('diagnose_cf07_last_index_latch_region_ownership_from_real_class_ir','cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir')
cut=setup.index('        fn find_if(');end=setup.index('        let mut budget =',cut);setup=setup[:cut]+setup[end:]
assert 'fn find_loop' in setup and 'recover(' in setup
setup+='''        let loop_region = recovered.regions.iter().find_map(|region| find_loop(region, 5))
            .expect("actual lastIndexOf loop header 5");
        let Region::Loop { header, body, for_header, gateway_origins, .. } = loop_region else { unreachable!() };
        assert!(for_header.is_none());
        assert_eq!(gateway_origins, &[25]);
        let Some(Region::If { branch_bci: 16, join: None, .. }) = body.last() else {
            panic!("expected actual direct returning If tail: {body:#?}");
        };
        let header_node = view.index_of(header).expect("actual header has normal-flow node");
'''
p=Path('/private/tmp/jarde-return-latch-boundary-tests-luna-v1/region-budget-tests.patch');patch=p.read_text();add='\n'.join(line[1:] for line in patch.splitlines() if line.startswith('+') and not line.startswith('+++'))+'\n'
setup+=add
setup+='''        let mut complete = steps(1 << 20);
        assert_eq!(real_walker!(complete).implicit_tail_latch_origin(header_node, body, None)
            .expect("complete direct proof"), Some(25));
    }
'''
p=root/'crates/jarde-java/src/region.rs';s=p.read_text();idx=s.rfind('\n}');s=s[:idx]+'\n'+setup+s[idx:];p.write_text(s)
