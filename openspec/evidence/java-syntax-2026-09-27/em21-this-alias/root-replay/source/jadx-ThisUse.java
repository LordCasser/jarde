package em21;

import java.util.Objects;

/* JADX INFO: loaded from: input.jar:em21/ThisUse.class */
public class ThisUse {
    public int field;
    private int touches;

    public void inline() {
        touch();
        this.field = 123;
    }

    public void checked() {
        if (Objects.isNull(this)) {
            System.out.println("null");
        }
        touch();
        this.field = 123;
    }

    public ThisUse choose() {
        ThisUse thisUse;
        if (this.field == 7) {
            thisUse = this;
            System.out.print("");
        } else {
            thisUse = new ThisUse();
        }
        thisUse.touch();
        return thisUse;
    }

    private void touch() {
        this.touches++;
    }

    public int touches() {
        return this.touches;
    }
}
