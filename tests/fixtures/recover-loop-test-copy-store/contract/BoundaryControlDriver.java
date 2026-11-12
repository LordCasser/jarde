public class BoundaryControlDriver {
    // Independent expected completion and event record for each deliberately different shape.
    static String[] expected(String target, int[] values, int failure) {
        StringBuilder trace = new StringBuilder("open;");
        if (failure == 4) return new String[]{"777","open",trace.toString(),"0"};
        if (target.equals("BoundaryOtherReceiver") || target.equals("BoundaryReassignedReceiver")) trace.append("open;");
        int stop = -1;
        for (int i=0;i<values.length;i++) if(values[i]==-1) {stop=i;break;}
        if(stop<0)throw new AssertionError("oracle inputs must terminate");
        int throwAt = failure==1 ? 0 : (failure==2 || failure==5) ? 1 : -1;
        boolean bodyFails = throwAt>=0 && throwAt<=stop;
        int reads = bodyFails ? throwAt+1 : stop+1;
        for(int i=0;i<reads;i++) trace.append("read").append(i).append(';');
        boolean closeFails = failure==3 || failure==5;
        String thrown = bodyFails ? "read" : "none";
        if(target.equals("BoundarySkippedNormalCleanup")) {
            if(bodyFails) {trace.append("close;");if(closeFails)thrown="close";}
        } else if(target.equals("BoundaryChangedCloseTarget") && bodyFails) {
            trace.append("finish;");
        } else {
            trace.append("close;");
            if(closeFails) {
                thrown="close";
                if(target.equals("BoundaryProtectedCleanup") && !bodyFails)trace.append("close;");
            } else if(target.equals("BoundaryChangedRethrow") && bodyFails)thrown="replacement";
        }
        int sum=0;for(int i=0;i<stop;i++)sum+=values[i];
        return new String[]{Integer.toString(thrown.equals("none")?sum:777),thrown,trace.toString(),Integer.toString(reads)};
    }

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
            if (actual!=null && actual!=read && actual!=close && actual!=open && actual!=BoundaryResource.replacement) throw new AssertionError("unexpected failure",actual);
            String[] expected = expected(target,inputs[which],failure);
            Throwable expectedFailure = expected[1].equals("none") ? null : expected[1].equals("read") ? read : expected[1].equals("close") ? close : expected[1].equals("open") ? open : BoundaryResource.replacement;
            if(actual!=expectedFailure || result!=Integer.parseInt(expected[0]) || !ContractResource.events.toString().equals(expected[2]) || ContractResource.next!=Integer.parseInt(expected[3]))throw new AssertionError(target+" "+which+"/"+failure+": independent expected record mismatch; actual="+result+"/"+actual+"/"+ContractResource.events+", expected="+java.util.Arrays.toString(expected));
            for(RuntimeException value:new RuntimeException[]{read,close,open}) {Throwable[] suppressed=value.getSuppressed();if(value.getCause()!=cause || suppressed.length!=2 || suppressed[0]!=(value==close?two:one) || suppressed[1]!=(value==close?one:two))throw new AssertionError("exception metadata");}
            System.out.println(which+"/"+failure+":result="+result+";throw="+(actual==null?"none":actual==read?"read":actual==close?"close":actual==BoundaryResource.replacement?"replacement":"open")+";trace="+ContractResource.events+";alias="+(AliasState.observed==null?"null":AliasState.observed==AliasState.first?"first":"second"));
        }
    }
}
