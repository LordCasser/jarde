public class ControlDriver {
    public static void main(String[] args) throws Exception {
        String target = args[0];
        java.lang.reflect.Method method = Class.forName(target).getMethod("run",String.class);
        int[][] inputs = { {-1,-1}, {0,-1}, {65,66,-1}, {-2,3,-1}, {13,-1}, {65,13,-1} };
        for (int which=0; which<inputs.length; which++) for (int failure=0;failure<6;failure++) {
            Throwable cause = new RuntimeException("cause"), one = new RuntimeException("one"), two = new RuntimeException("two");
            RuntimeException read = new RuntimeException("read",cause), close = new RuntimeException("close",cause), open = new RuntimeException("open",cause);
            read.addSuppressed(one);read.addSuppressed(two);close.addSuppressed(two);close.addSuppressed(one);open.addSuppressed(one);open.addSuppressed(two);
            ContractResource.values=inputs[which];ContractResource.next=0;ContractResource.throwAt=failure==1?0:(failure==2||failure==5)?1:-1;
            ContractResource.events=new StringBuilder();ContractResource.readFailure=read;ContractResource.closeFailure=(failure==3||failure==5)?close:null;ContractResource.openFailure=failure==4?open:null;
            AliasState.observed=null;
            int result=777;Throwable actual=null;
            try { result=((Integer)method.invoke(null,"input")).intValue(); } catch(java.lang.reflect.InvocationTargetException wrapped) {actual=wrapped.getCause();}
            if (actual!=null && actual!=read && actual!=close && actual!=open) throw new AssertionError("unexpected failure",actual);
            if (failure==4 && actual!=open || (failure==3||failure==5) && actual!=close)throw new AssertionError("completion replacement");
            for(RuntimeException value:new RuntimeException[]{read,close,open}) {Throwable[] suppressed=value.getSuppressed();if(value.getCause()!=cause || suppressed.length!=2 || suppressed[0]!=(value==close?two:one) || suppressed[1]!=(value==close?one:two))throw new AssertionError("exception metadata");}
            if(target.equals("AliasFinallyCopy")) {
                Object expected = failure==4?null:failure==1 || inputs[which][0]==-1 || inputs[which][0]==13 ? AliasState.first:AliasState.second;
                if(AliasState.observed!=expected)throw new AssertionError("throwing RHS must preserve old alias");
            }
            System.out.println(which+"/"+failure+":result="+result+";throw="+(actual==null?"none":actual==read?"read":actual==close?"close":"open")+";trace="+ContractResource.events+";alias="+(AliasState.observed==null?"null":AliasState.observed==AliasState.first?"first":"second"));
        }
    }
}
