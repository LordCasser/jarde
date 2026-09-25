package defpackage;

/* JADX INFO: loaded from: Stage.class */
enum Stage {
    START(4),
    FINISH(9);

    final int ordinalValue;

    Stage(int i) {
        this.ordinalValue = i;
    }

    int code() {
        return this.ordinalValue;
    }
}
