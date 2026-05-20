import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import SidebarCard from "../components/SidebarCard";

describe("SidebarCard", () => {
  it("renders the label", () => {
    render(<SidebarCard icon="/tv.svg" label="Live TV" />);
    expect(screen.getByText("Live TV")).toBeInTheDocument();
  });

  it("renders an img with the given icon src", () => {
    const { container } = render(<SidebarCard icon="/tv.svg" label="Live TV" />);
    // alt="" makes this a presentation role; query directly
    expect(container.querySelector("img")).toHaveAttribute("src", "/tv.svg");
  });

  it("calls onClick when clicked", async () => {
    const handler = vi.fn();
    render(<SidebarCard icon="/tv.svg" label="Live TV" onClick={handler} />);
    await userEvent.click(screen.getByText("Live TV"));
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it("renders without crashing when onClick is omitted", () => {
    render(<SidebarCard icon="/tv.svg" label="Movies" />);
    expect(screen.getByText("Movies")).toBeInTheDocument();
  });
});
