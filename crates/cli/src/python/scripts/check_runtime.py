def main():
    try:
        # Check if zygo is available
        import zygo
    except ImportError:
        print("zygo is not available")
        raise

if __name__ == "__main__":
    main()
