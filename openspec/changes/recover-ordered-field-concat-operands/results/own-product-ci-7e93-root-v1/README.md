# Own-CI observation archive

This archive records an accepted local own-CI observation for product commit `7e93eb30647338dd8a6ddfc9994392f23457aa3e` and run `38297425449`. It contains the verifier acceptance, its three-command capture and pinned raw/log artifacts, and the referenced verifier tools. The prior workspace-v9, local, and full-class archives are referenced and byte-checked; their contents are not duplicated here. This records the accepted observation only; it does not claim that the product was pushed or merged.

Verify `capture.tar.gz` against `members.json`; every member is reopened and checked by the preparation script.

`verifier-history.tar.gz` and `verifier-history-members.json` preserve the actual successful v4 invocation and the v3/v4 adapter changes. Two preceding private verifier runs failed because of adapter assumptions (workspace planner API and auxiliary parser membership); their failures were observed in root tool output, not independently captured raw streams. The compiled 4854-source closure and production bytes remain unchanged. This history is separate from the immutable 16-member accepted CI capture.
