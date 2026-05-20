import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import PlayButton from "../components/PlayButton";

describe("PlayButton", () => {
  it("renders the label", () => {
    render(<PlayButton onClick={() => {}} label="Play" />);
    expect(screen.getByRole("button")).toHaveTextContent("Play");
  });

  it("calls onClick when clicked", async () => {
    const handler = vi.fn();
    render(<PlayButton onClick={handler} label="Play" />);
    await userEvent.click(screen.getByRole("button"));
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it("hides the progress bar when progressPct is 0", () => {
    const { container } = render(<PlayButton onClick={() => {}} label="Play" progressPct={0} />);
    // The progress bar inner div only exists when progressPct > 0
    const bars = container.querySelectorAll('[style*="background: rgb(229, 9, 20)"]');
    expect(bars).toHaveLength(0);
  });

  it("shows the progress bar when progressPct > 0", () => {
    const { container } = render(<PlayButton onClick={() => {}} label="Resume" progressPct={42} />);
    const bar = container.querySelector('[style*="width: 42%"]');
    expect(bar).toBeInTheDocument();
  });
});
