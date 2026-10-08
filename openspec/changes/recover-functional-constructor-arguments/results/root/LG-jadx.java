package defpackage;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.PriorityQueue;
import java.util.Stack;

/* JADX INFO: loaded from: lg.jar:LG.class */
public class LG {
    static int[] pqLambda(int[] iArr) {
        PriorityQueue priorityQueue = new PriorityQueue((num, num2) -> {
            return num2.intValue() - num.intValue();
        });
        for (int i : iArr) {
            priorityQueue.offer(Integer.valueOf(i));
        }
        int[] iArr2 = new int[iArr.length];
        for (int i2 = 0; i2 < iArr2.length; i2++) {
            iArr2[i2] = ((Integer) priorityQueue.poll()).intValue();
        }
        return iArr2;
    }

    static int[] stackOps(int[] iArr) {
        Stack stack = new Stack();
        for (int i : iArr) {
            stack.push(Integer.valueOf(i));
        }
        return new int[]{((Integer) stack.peek()).intValue(), ((Integer) stack.pop()).intValue(), stack.size()};
    }

    static List<Integer> dequeOps(int[] iArr) {
        ArrayDeque arrayDeque = new ArrayDeque();
        for (int i : iArr) {
            arrayDeque.addFirst(Integer.valueOf(i));
        }
        arrayDeque.addLast(99);
        ArrayList arrayList = new ArrayList();
        arrayList.add((Integer) arrayDeque.peekFirst());
        arrayList.add((Integer) arrayDeque.peekLast());
        arrayList.add(Integer.valueOf(arrayDeque.size()));
        return arrayList;
    }

    protected void finalize() throws Throwable {
        try {
            System.out.print("fin;");
        } finally {
            super.finalize();
        }
    }

    public static void main(String[] strArr) {
        System.out.println("" + Arrays.toString(pqLambda(new int[]{3, 1, 2})) + "/" + Arrays.toString(stackOps(new int[]{7, 8})) + "/" + dequeOps(new int[]{5, 6}));
        Runtime.getRuntime().runFinalization();
    }
}
