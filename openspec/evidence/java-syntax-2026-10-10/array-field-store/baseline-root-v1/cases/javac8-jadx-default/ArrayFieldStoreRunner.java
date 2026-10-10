package defpackage;

import java.util.Arrays;

public final class ArrayFieldStoreRunner {
    public static void main(String[] args) {
        ArrayFieldStore target = new ArrayFieldStore();
        target.storeLiteral();
        byte[] beforeSuccess = target.readValues();
        ArrayFieldStore.resetTrace();
        ArrayFieldStore.replace(target, 0);
        System.out.println("success=" + Arrays.toString(target.readValues())
                + ":fresh=" + (target.readValues() != beforeSuccess)
                + ":trace=" + ArrayFieldStore.readTrace());

        target.storeLiteral();
        byte[] beforeFailure = target.readValues();
        ArrayFieldStore.resetTrace();
        try {
            ArrayFieldStore.replace(target, 2);
            System.out.println("failure=missing");
        } catch (IllegalStateException error) {
            System.out.println("failure=" + error.getClass().getSimpleName()
                    + ":" + error.getMessage()
                    + ":same=" + (target.readValues() == beforeFailure)
                    + ":values=" + Arrays.toString(target.readValues())
                    + ":trace=" + ArrayFieldStore.readTrace());
        }

        ArrayFieldStore.resetTrace();
        try {
            ArrayFieldStore.replace(null, 0);
            System.out.println("null=missing");
        } catch (NullPointerException error) {
            System.out.println("null=" + error.getClass().getSimpleName()
                    + ":trace=" + ArrayFieldStore.readTrace());
        }

        ArrayFieldStore.resetTrace();
        try {
            ArrayFieldStore.replace(null, 2);
            System.out.println("null-failure=missing");
        } catch (RuntimeException error) {
            System.out.println("null-failure=" + error.getClass().getSimpleName()
                    + ":" + error.getMessage()
                    + ":trace=" + ArrayFieldStore.readTrace());
        }
    }
}
