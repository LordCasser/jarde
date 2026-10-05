public class BT {
    static class Node { int val; Node left, right; Node(int v){ val = v; } }
    static Node root;
    static Node insert(Node arg0, int arg1){
        if (arg0 == null) { return new Node(arg1); }
        else if (arg1 < arg0.val) { arg0.left = insert(arg0.left, arg1); }
        else if (arg1 > arg0.val) { arg0.right = insert(arg0.right, arg1); }
        return arg0;
    }
    static boolean contains(Node arg0, int arg1){
        if (arg0 == null) { return false; }
        else if (arg1 == arg0.val) { return true; }
        else if (arg1 < arg0.val) { return contains(arg0.left, arg1); }
        else { return contains(arg0.right, arg1); }
    }
    static String inorder(Node arg0){
        if (arg0 == null) { return ""; }
        else { return new StringBuilder().append(inorder(arg0.left)).append(arg0.val).append(",").append(inorder(arg0.right)).toString(); }
    }
    static void add(int arg0){ BT.root = insert(BT.root, arg0); return; }
    public static void main(String[] a){
        int[] xs = {5, 3, 8, 1, 4, 7, 9, 2, 6};
        for(int x : xs){ add(x); }
        System.out.println(""+inorder(root)+"/"+contains(root,4)+"/"+contains(root,10));
    }
}
