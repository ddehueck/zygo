const RAW_COLOR_PATTERN =
  /#[\da-f]{3,8}\b|\b(?:rgb|rgba|hsl|hsla|hwb|oklch|oklab|lab|lch|color|color-mix)\s*\(/i;
const ARBITRARY_COLOR_UTILITY_PATTERN =
  /(?:^|[\s:])(?:bg|text|border(?:-[xytrblse])?|divide(?:-[xy])?|outline|ring(?:-offset)?|shadow|accent|caret|decoration|fill|stroke|from|via|to)-\[[^\]]+\]/;
const DEFAULT_PALETTE_UTILITY_PATTERN =
  /(?:^|[\s:])(?:bg|text|border(?:-[xytrblse])?|divide(?:-[xy])?|outline|ring(?:-offset)?|shadow|accent|caret|decoration|fill|stroke|from|via|to)-(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-(?:50|100|200|300|400|500|600|700|800|900|950)(?:\/[\w.]+)?!?(?=\s|$)/;
const COLOR_STYLE_PROPERTY_PATTERN =
  /^(?:color|background(?:Color)?|border(?:Top|Right|Bottom|Left)?Color|outlineColor|textDecorationColor|textEmphasisColor|caretColor|accentColor|columnRuleColor|fill|stroke|floodColor|lightingColor|stopColor)$/;

function checkString(context, node, value) {
  if (RAW_COLOR_PATTERN.test(value)) {
    context.report({ node, messageId: "rawColor" });
    return;
  }

  if (ARBITRARY_COLOR_UTILITY_PATTERN.test(value)) {
    context.report({ node, messageId: "arbitraryColor" });
    return;
  }

  if (DEFAULT_PALETTE_UTILITY_PATTERN.test(value)) {
    context.report({ node, messageId: "defaultPaletteColor" });
  }
}

function propertyName(property) {
  if (!property.computed && property.key.type === "Identifier") {
    return property.key.name;
  }

  if (property.key.type === "Literal" && typeof property.key.value === "string") {
    return property.key.value;
  }

  return undefined;
}

const noRawColors = {
  meta: {
    type: "problem",
    docs: {
      description: "Require semantic theme colors in JavaScript, TypeScript, and JSX.",
    },
    messages: {
      arbitraryColor: "Use a semantic theme color utility instead of an arbitrary color utility.",
      defaultPaletteColor:
        "Tailwind default palette colors are forbidden; use a semantic theme color utility.",
      inlineColor: "Inline color styles are forbidden; use a semantic Tailwind color utility.",
      rawColor: "Use a semantic theme color instead of a literal color value.",
    },
    schema: [],
  },
  create(context) {
    return {
      Literal(node) {
        if (typeof node.value === "string") {
          checkString(context, node, node.value);
        }
      },
      TemplateElement(node) {
        checkString(context, node, node.value.raw);
      },
      JSXAttribute(node) {
        if (
          node.name.type !== "JSXIdentifier" ||
          node.name.name !== "style" ||
          node.value?.type !== "JSXExpressionContainer" ||
          node.value.expression.type !== "ObjectExpression"
        ) {
          return;
        }

        for (const property of node.value.expression.properties) {
          if (
            property.type === "Property" &&
            COLOR_STYLE_PROPERTY_PATTERN.test(propertyName(property) ?? "")
          ) {
            context.report({ node: property, messageId: "inlineColor" });
          }
        }
      },
    };
  },
};

export default {
  meta: {
    name: "theme-colors",
  },
  rules: {
    "no-raw-colors": noRawColors,
  },
};
