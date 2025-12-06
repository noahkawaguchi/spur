-- Reusable type for non-empty text columns
CREATE DOMAIN non_empty_text AS TEXT
    CONSTRAINT text_non_empty CHECK (VALUE ~ '\S'); -- At least one non-whitespace character
