public class ReturnSinks {
 public int value;
 public int post(){return value++;}
 public int pre(){return ++value;}
 public static int guarded(Object lock,int value){synchronized(lock){return value;}}
 public static int choice(boolean which,int left,int right){return which?left:right;}
}
