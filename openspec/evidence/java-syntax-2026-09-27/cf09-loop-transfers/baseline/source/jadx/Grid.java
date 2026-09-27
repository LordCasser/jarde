
/* JADX INFO: loaded from: Grid.jar:Grid.class */
public class Grid {
    static int nestedBreak(int i) {
        int i2 = 0;
        for (int i3 = 0; i3 < i; i3++) {
            for (int i4 = 0; i4 < i && i3 + i4 <= 5; i4++) {
                i2++;
            }
        }
        return i2;
    }

    static int labeledContinue(int i) {
        int i2 = 0;
        for (int i3 = 0; i3 < i; i3++) {
            for (int i4 = 0; i4 < i && i4 != 2; i4++) {
                i2++;
            }
        }
        return i2;
    }

    static int labeledBreak(int i) {
        int i2 = 0;
        loop0: for (int i3 = 0; i3 < i; i3++) {
            for (int i4 = 0; i4 < i; i4++) {
                if (i4 == 3) {
                    break loop0;
                }
                i2++;
            }
            i2 += 10;
        }
        return i2;
    }
}
