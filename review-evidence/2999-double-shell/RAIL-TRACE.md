# Rail creation trace and controls

The two tangent open-wall bridge pairs are now traced before and after the rail/centre-line manicuring. Both rails of both pairs still span z=1.9999999999999996..30. The centre line is trimmed to those same old heights. The intermediate output is byte-identical to the first capture. This establishes that the wrong extent is already present when the bridge is created.

Source order: do_extended_offset processes tangent pairs first, constructs these bridges through mapped_edge_prefix, and marks them so its later general mapped-edge pass skips them. The neighbouring non-tangent floor/rim mitres occur in that later pass. Their final boundaries are z=1.5 and29.5, while the previously created bridge trims keep the old height span. A later bridge phase or explicit endpoint dependency is a candidate design, not an implemented or verified repair. The construction owners have been asked where it belongs.

The private trace is cfg(test) printing only. It was removed with exact original source hashes verified for both files. The exact removed patches and receipts are beside this file. Initial topology-cache checks are reported accurately in the earlier evidence; after cache refresh integrity passes, while the intermediate remains nonmanifold.

Public-API controls use the same frozen main production rlib, without either private test patch. The reported face is refused at distances0.25,0.5,0.75. All four inner flat walls are refused at0.5. All four outer walls, the bottom, rim and inner floor return manifold bodies; their measured volumes are diagnostic only and have not been independently graded. Neither status nor those volumes are presented as full correctness certificates.
