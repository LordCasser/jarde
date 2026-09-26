package checked;

import java.io.IOException;

/* JADX INFO: loaded from: CheckedParent.class */
class CheckedParent {
    CheckedParent() {
    }

    /* JADX INFO: Access modifiers changed from: package-private */
    public String select(Number number) {
        return "no-checked-exception";
    }

    String select(Integer num) throws IOException {
        return "checked-exception";
    }
}
