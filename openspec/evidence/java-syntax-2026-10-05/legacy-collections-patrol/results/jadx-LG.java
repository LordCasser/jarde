package defpackage;

/* JADX INFO: loaded from: LG.class */
public class LG {
    static int[] pqLambda(int[] iArr) {
        java.util.PriorityQueue priorityQueue = new java.util.PriorityQueue((num, num2) -> {
            return num2.intValue() - num.intValue();
        });
        for (int i : iArr) {
            priorityQueue.offer(java.lang.Integer.valueOf(i));
        }
        int[] iArr2 = new int[iArr.length];
        for (int i2 = 0; i2 < iArr2.length; i2++) {
            iArr2[i2] = ((java.lang.Integer) priorityQueue.poll()).intValue();
        }
        return iArr2;
    }

    static int[] stackOps(int[] iArr) {
        java.util.Stack stack = new java.util.Stack();
        for (int i : iArr) {
            stack.push(java.lang.Integer.valueOf(i));
        }
        return new int[]{((java.lang.Integer) stack.peek()).intValue(), ((java.lang.Integer) stack.pop()).intValue(), stack.size()};
    }

    static java.util.List<java.lang.Integer> dequeOps(int[] iArr) {
        java.util.ArrayDeque arrayDeque = new java.util.ArrayDeque();
        for (int i : iArr) {
            arrayDeque.addFirst(java.lang.Integer.valueOf(i));
        }
        arrayDeque.addLast(99);
        java.util.ArrayList arrayList = new java.util.ArrayList();
        arrayList.add((java.lang.Integer) arrayDeque.peekFirst());
        arrayList.add((java.lang.Integer) arrayDeque.peekLast());
        arrayList.add(java.lang.Integer.valueOf(arrayDeque.size()));
        return arrayList;
    }

    protected void finalize() throws java.lang.Throwable {
        try {
            java.lang.System.out.print("fin;");
        } finally {
            super.finalize();
        }
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + java.util.Arrays.toString(pqLambda(new int[]{3, 1, 2})) + "/" + java.util.Arrays.toString(stackOps(new int[]{7, 8})) + "/" + dequeOps(new int[]{5, 6}));
        java.lang.Runtime.getRuntime().runFinalization();
    }
}
