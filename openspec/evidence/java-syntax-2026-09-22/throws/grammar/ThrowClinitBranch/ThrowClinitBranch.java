public class ThrowClinitBranch {static {if(System.nanoTime()==0L)throw new IllegalArgumentException("a");else if(true)throw new IllegalStateException("b");}}
