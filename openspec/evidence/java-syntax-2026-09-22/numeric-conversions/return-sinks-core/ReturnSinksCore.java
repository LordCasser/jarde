public class ReturnSinksCore {
 public int value;
 public int post(){return value++;}
 public int pre(){return ++value;}
 public static int guarded(Object lock,int value){synchronized(lock){return value;}}
}
