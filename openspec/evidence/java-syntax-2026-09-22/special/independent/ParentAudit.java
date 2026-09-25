package probe;
public class ParentAudit {
 public static int count;
 public int read(){return 3;}
 public void add(int x){count=count+x;}
 public static void bump(int x){count=count+x;}
 public static int argument(){count=count+1;return 4;}
}
