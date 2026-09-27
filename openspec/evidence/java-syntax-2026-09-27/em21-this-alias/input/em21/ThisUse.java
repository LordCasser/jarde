package em21;

import java.util.Objects;

public class ThisUse {
    public int field;
    private int touches;

    public void inline() {
        ThisUse something = this;
        something.touch();
        something.field = 123;
    }

    public void checked() {
        ThisUse thisVar = this;
        if (Objects.isNull(thisVar)) {
            System.out.println("null");
        }
        thisVar.touch();
        thisVar.field = 123;
    }

    public ThisUse choose() {
        ThisUse res;
        if (field == 7) {
            res = this;
            System.out.print("");
        } else {
            res = new ThisUse();
        }
        res.touch();
        return res;
    }

    private void touch() {
        touches++;
    }

    public int touches() {
        return touches;
    }
}
