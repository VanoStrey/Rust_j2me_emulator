package fixtures;

import javax.microedition.midlet.MIDlet;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.game.GameCanvas;

public class Fixture5TwoGameCanvases extends MIDlet {
    public static class CanvasA extends GameCanvas {
        public CanvasA() { super(false); }
    }
    public static class CanvasB extends GameCanvas {
        public CanvasB() { super(false); }
    }

    protected void startApp() {
        CanvasA ca = new CanvasA();
        CanvasB cb = new CanvasB();

        Graphics ga = ca.getGraphics();
        ga.setColor(0x00FF0000); // Red
        ga.fillRect(10, 10, 30, 30);

        Graphics gb = cb.getGraphics();
        gb.setColor(0x000000FF); // Blue
        gb.fillRect(50, 50, 30, 30);

        Display.getDisplay(this).setCurrent(ca);
        ca.flushGraphics();
    }
    protected void pauseApp() {}
    protected void destroyApp(boolean u) {}
}
