<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Visitor;

/**
 * NodeContext represents the context information for a node during HTML traversal.
 *
 * This object is passed to all visitor methods and provides information about
 * the current node, its position in the document, and its relationship to parent nodes.
 *
 * @phpstan-type NodeContextArray array{
 *     node_type: string,
 *     tag_name: string,
 *     attributes: array<string, string>,
 *     depth: int,
 *     index_in_parent: int,
 *     parent_tag: string|null,
 *     is_inline: bool,
 * }
 */
final class NodeContext
{
    /**
     * @param array<string, string> $attributes
     */
    public function __construct(
        public readonly string $nodeType,
        public readonly string $tagName,
        public readonly array $attributes,
        public readonly int $depth,
        public readonly int $indexInParent,
        public readonly string|null $parentTag,
        public readonly bool $isInline,
    ) {
    }

    /**
     * Create a NodeContext from an associative array (from Rust FFI).
     *
     * @param array<string, mixed> $data
     * @phpstan-assert NodeContextArray $data
     */
    public static function fromArray(array $data): self
    {
        $nodeType = '';
        if (isset($data['node_type']) && \is_string($data['node_type'])) {
            $nodeType = $data['node_type'];
        }

        $tagName = '';
        if (isset($data['tag_name']) && \is_string($data['tag_name'])) {
            $tagName = $data['tag_name'];
        }

        $attributes = [];
        if (isset($data['attributes']) && \is_array($data['attributes'])) {
            $attributes = $data['attributes'];
        }

        $depth = 0;
        if (isset($data['depth']) && \is_int($data['depth'])) {
            $depth = $data['depth'];
        }

        $indexInParent = 0;
        if (isset($data['index_in_parent']) && \is_int($data['index_in_parent'])) {
            $indexInParent = $data['index_in_parent'];
        }

        $parentTag = null;
        if (isset($data['parent_tag']) && \is_string($data['parent_tag'])) {
            $parentTag = $data['parent_tag'];
        }

        $isInline = false;
        if (isset($data['is_inline']) && \is_bool($data['is_inline'])) {
            $isInline = $data['is_inline'];
        }

        return new self(
            nodeType: $nodeType,
            tagName: $tagName,
            attributes: $attributes,
            depth: $depth,
            indexInParent: $indexInParent,
            parentTag: $parentTag,
            isInline: $isInline,
        );
    }

    /**
     * Convert NodeContext to an associative array for passing to Rust FFI.
     *
     * @return array<string, mixed>
     * @phpstan-return NodeContextArray
     */
    public function toArray(): array
    {
        return [
            'node_type' => $this->nodeType,
            'tag_name' => $this->tagName,
            'attributes' => $this->attributes,
            'depth' => $this->depth,
            'index_in_parent' => $this->indexInParent,
            'parent_tag' => $this->parentTag,
            'is_inline' => $this->isInline,
        ];
    }
}
