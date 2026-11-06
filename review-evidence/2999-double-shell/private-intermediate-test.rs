
    // BEGIN OWNED TEMPORARY ISSUE 2954 DIAGNOSTIC
    #[test]
    #[ignore = "private intermediate-offset diagnosis"]
    fn issue_2954_inspect_intermediate_offset() {
        let mut k = Kernel { fixes: crate::kernel::UpstreamFixes::all(), ..Kernel::default() };
        let mut attrs = AttributeStore::default();
        let mut records = crate::topology::topology::NotifyRecords::new();
        let mut env = crate::topology::brep::p10::TopologyEnv { all_attributes: &[], attributes: &mut attrs, key_of: &crate::topology::loopuse::notify_ref, records: &mut records, next_key: 1 << 50 };
        let bytes = std::fs::read("/tmp/solidcore-2954-evidence/second-shell-input.smb").unwrap();
        let mut input = crate::topology::api_files::read_brep_from_file(&mut k, Some(bytes), true, false, &mut env).unwrap();
        input.set_key(original_key(EntityRef::Brep(BrepId::from_index(0))).key);
        let open = input.face_list()[14];
        let options = ShellOptions { offset: OffsetOptions { distance: -0.5, extend_convex_edges: true, merge_results: true }, create_offset_solid: true, merge_sheet_side_walls: true };
        let mut fixture = NotificationFixture::new(&input);
        let mut offset = run_shell_offset(&mut k, &mut input, options, &[open], &mut fixture.context()).unwrap();
        eprintln!("ISSUE2954 offset: faces={} edges={} vertices={} manifold={:?}", offset.face_list().len(), offset.edge_list().len(), offset.vertex_list().len(), brep_is_manifold_solid(&offset));
        let env = crate::topology::brep_data_write::FromBrepEnv { attributes: &fixture.attributes, records: &fixture.records, key_of: &result_key, curve_key: &|e| result_curve(e, 0).key, uv_curve_key: &|_| ObjectKey(0), surface_key: &|_| ObjectKey(0) };
        let bytes = crate::topology::api_files::write_brep_to_file(&k, Some(&offset), &env, true).unwrap();
        std::fs::write("/tmp/solidcore-2954-evidence/intermediate-offset.smb", bytes).unwrap();
        eprintln!("ISSUE2954 offset integrity: {:?}", crate::topology::integrity::check(&offset));
        eprintln!("ISSUE2954 offset volume: {:?}", crate::mass::integrator::properties::brep_volume_default(&k, &mut offset, 1e-8));
    }
    // END OWNED TEMPORARY ISSUE 2954 DIAGNOSTIC
