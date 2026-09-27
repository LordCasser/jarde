package em21;

public class ThisAliasRefusals {
    public int value;
    public int touches;

    public void target() {
        touches++;
    }

    public static void consume(ThisAliasRefusals value) {
    }

    public ThisAliasRefusals reassign(boolean replace) {
        ThisAliasRefusals local = this;
        if (replace) {
            local = new ThisAliasRefusals();
        }
        local.target();
        return local;
    }

    public void argument() {
        ThisAliasRefusals local = this;
        consume(local);
    }

    public ThisAliasRefusals returnAlias() {
        ThisAliasRefusals local = this;
        return local;
    }

    public int readField() {
        ThisAliasRefusals local = this;
        return local.value;
    }

    public void reusedSlot() {
        {
            ThisAliasRefusals local = this;
            local.target();
        }
        {
            int local = 1;
            value = local;
        }
    }

    public void unusedAlias() {
        ThisAliasRefusals local = this;
    }
}
