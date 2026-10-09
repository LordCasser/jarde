package defpackage;

import java.util.HashMap;
import java.util.Map;

/* JADX INFO: loaded from: RawBoundWriter.jar:RawBoundWriter.class */
public class RawBoundWriter {
    public Map<String, String> v;

    public <R extends HashMap> void put(R r, boolean z) {
        this.v = r;
    }
}
