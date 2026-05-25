import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import ContextMenu from "../components/ContextMenu";

describe("ContextMenu", () => {
  it("renders all item labels", () => {
    const items = [
      { label: "Play", onClick: vi.fn() },
      { label: "Delete", onClick: vi.fn() },
    ];
    render(<ContextMenu x={10} y={20} items={items} onClose={vi.fn()} />);
    expect(screen.getByText("Play")).toBeInTheDocument();
    expect(screen.getByText("Delete")).toBeInTheDocument();
  });

  it("calls the item onClick and onClose when an item is clicked", async () => {
    const itemClick = vi.fn();
    const onClose = vi.fn();
    render(
      <ContextMenu
        x={10}
        y={20}
        items={[{ label: "Play", onClick: itemClick }]}
        onClose={onClose}
      />
    );
    await userEvent.click(screen.getByText("Play"));
    expect(itemClick).toHaveBeenCalledTimes(1);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("calls onClose when Escape is pressed", async () => {
    const onClose = vi.fn();
    render(
      <ContextMenu
        x={10}
        y={20}
        items={[{ label: "Play", onClick: vi.fn() }]}
        onClose={onClose}
      />
    );
    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("calls onClose when clicking outside the menu", async () => {
    const onClose = vi.fn();
    render(
      <div>
        <ContextMenu
          x={10}
          y={20}
          items={[{ label: "Play", onClick: vi.fn() }]}
          onClose={onClose}
        />
        <button>Outside</button>
      </div>
    );
    await userEvent.click(screen.getByText("Outside"));
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
