package defpackage;

/* JADX INFO: loaded from: MaskCondition.class.jar:MaskCondition.class */
public class MaskCondition {
    public int method3(int i, int i2) {
        if (i + i2 < 10) {
            return i;
        }
        return (i & i2) != 0 ? i * i2 : i2;
    }
}
