package defpackage;

/* JADX INFO: loaded from: new-input.jar:EffectsBase.class */
class EffectsBase {
    EffectsBase() {
    }

    /* JADX INFO: Access modifiers changed from: package-private */
    public int combine(int left, int right) {
        OuterSuperEffects.events = (OuterSuperEffects.events * 10) + 3;
        return (left * 10) + right;
    }
}
