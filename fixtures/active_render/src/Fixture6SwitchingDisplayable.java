package fixtures;

import javax.microedition.midlet.MIDlet;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.game.GameCanvas;

public class Fixture6SwitchingDisplayable extends MIDlet {
    public static class Screen1 extends GameCanvas {
        public Screen1() { super(false); }
    }
    public static class Screen2 extends GameCanvas {
        public Screen2() { super(false); }
    }

    protected void startApp() {
        Screen1 s1 = new Screen1();
        Screen2 s2 = new Screen2();

        Display disp = Display.getDisplay(this);
        disp.setCurrent(s1);

        Graphics g1 = s1.getGraphics();
        g1.setColor(0x00FF0000); // Red
        g1.fillRect(0, 0, 240, 320);
        s1.flushGraphics();

        // Now switch to s2
        disp.setCurrent(s2);
        Graphics g2 = s2.getGraphics();
        g2.setColor(0x0000FF00); // Green
        g2.fillRect(0, 0, 240, 320);
        s2.flushGraphics();

        // Inactive s1 attempts to flush! Must NOT overwrite active s2!
        g1.setColor(0x000000FF); // Blue
        g1.fillRect(0, 0, 240, 320);
        s1.flushGraphics();
    }
    protected void pauseApp() {}
    protected void destroyApp(boolean u) {}
}
