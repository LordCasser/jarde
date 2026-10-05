public class BT {
    static class Node { int val; Node left, right; Node(int v){ val = v; } }
    static Node root;
    static Node insert(Node n, int v){                       // 递归结果写回字段
        if(n == null){ return new Node(v); }
        if(v < n.val){ n.left = insert(n.left, v); }
        else if(v > n.val){ n.right = insert(n.right, v); }
        return n;
    }
    static boolean contains(Node n, int v){                  // 递归只读
        if(n == null){ return false; }
        if(v == n.val){ return true; }
        return v < n.val ? contains(n.left, v) : contains(n.right, v);
    }
    static String inorder(Node n){                           // 递归累积
        if(n == null){ return ""; }
        return inorder(n.left) + n.val + "," + inorder(n.right);
    }
    static void add(int v){ root = insert(root, v); }        // 静态字段根更新
    public static void main(String[] a){
        int[] xs = {5, 3, 8, 1, 4, 7, 9, 2, 6};
        for(int x : xs){ add(x); }
        System.out.println(""+inorder(root)+"/"+contains(root,4)+"/"+contains(root,10));
    }
}
